use super::{State, DEVICE_ID_LENGTH, TOKEN_LENGTH};
use crate::{utils, ConduitResult, Database, Error, Ruma};
use ruma::{
    api::client::{
        error::ErrorKind,
        r0::session::{get_login_types, login, logout, logout_all},
    },
    UserId,
};

#[cfg(feature = "conduit_bin")]
use rocket::{get, post};

/// # `GET /_matrix/client/r0/login`
///
/// Get the homeserver's supported login types. One of these should be used as the `type` field
/// when logging in.
#[cfg_attr(feature = "conduit_bin", get("/_matrix/client/r0/login"))]
pub fn get_login_types_route() -> ConduitResult<get_login_types::Response> {
    Ok(get_login_types::Response::new(vec![get_login_types::LoginType::Password]).into())
}

/// # `POST /_matrix/client/r0/login`
///
/// Authenticates the user and returns an access token it can use in subsequent requests.
///
/// - The returned access token is associated with the user and device
/// - Old access tokens of that device should be invalidated
/// - If `device_id` is unknown, a new device will be created
///
/// Note: You can use [`GET /_matrix/client/r0/login`](fn.get_supported_versions_route.html) to see
/// supported login types.
#[cfg_attr(
    feature = "conduit_bin",
    post("/_matrix/client/r0/login", data = "<body>")
)]
pub fn login_route(
    db: State<'_, Database>,
    body: Ruma<login::Request<'_>>,
) -> ConduitResult<login::Response> {
    // Validate login method
    let user_id =
        // TODO: Other login methods
        if let (login::IncomingUserInfo::MatrixId(username), login::IncomingLoginInfo::Password { password }) =
            (&body.user, &body.login_info)
        {
            let user_id = UserId::parse_with_server_name(username.to_string(), db.globals.server_name())
                .map_err(|_| Error::BadRequest(
                    ErrorKind::InvalidUsername,
                    "Username is invalid."
                ))?;
            let hash = db.users.password_hash(&user_id)?
                .ok_or(Error::BadRequest(
                    ErrorKind::Forbidden,
                    "Wrong username or password."
                ))?;

            if hash.is_empty() {
                return Err(Error::BadRequest(
                    ErrorKind::UserDeactivated,
                    "The user has been deactivated"
                ));
            }

            let hash_matches =
                argon2::verify_encoded(&hash, password.as_bytes()).unwrap_or(false);

            if !hash_matches {
                return Err(Error::BadRequest(ErrorKind::Forbidden, "Wrong username or password."));
            }

            user_id
        } else {
            return Err(Error::BadRequest(ErrorKind::Forbidden, "Bad login type."));
        };

    // Generate new device id if the user didn't specify one
    let device_id = body
        .body
        .device_id
        .clone()
        .unwrap_or_else(|| utils::random_string(DEVICE_ID_LENGTH).into());

    // Generate a new token for the device
    let token = utils::random_string(TOKEN_LENGTH);

    // TODO: Don't always create a new device
    // Add device
    db.users.create_device(
        &user_id,
        &device_id,
        &token,
        body.initial_device_display_name.clone(),
    )?;

    Ok(login::Response {
        user_id,
        access_token: token,
        home_server: Some(db.globals.server_name().to_owned()),
        device_id,
        well_known: None,
    }
    .into())
}

/// # `POST /_matrix/client/r0/logout`
///
/// Log out the current device.
///
/// - Invalidates the access token
/// - Deletes the device and most of it's data (to-device events, last seen, etc.)
#[cfg_attr(
    feature = "conduit_bin",
    post("/_matrix/client/r0/logout", data = "<body>")
)]
pub fn logout_route(
    db: State<'_, Database>,
    body: Ruma<logout::Request>,
) -> ConduitResult<logout::Response> {
    let sender_user = body.sender_user.as_ref().expect("user is authenticated");
    let sender_device = body.sender_device.as_ref().expect("user is authenticated");

    db.users.remove_device(&sender_user, sender_device)?;

    Ok(logout::Response::new().into())
}

/// # `POST /_matrix/client/r0/logout/all`
///
/// Log out all devices of this user.
///
/// - Invalidates all access tokens
/// - Deletes devices and most of their data (to-device events, last seen, etc.)
///
/// Note: This is equivalent to calling [`GET /_matrix/client/r0/logout`](fn.logout_route.html)
/// from each device of this user.
#[cfg_attr(
    feature = "conduit_bin",
    post("/_matrix/client/r0/logout/all", data = "<body>")
)]
pub fn logout_all_route(
    db: State<'_, Database>,
    body: Ruma<logout_all::Request>,
) -> ConduitResult<logout_all::Response> {
    let sender_user = body.sender_user.as_ref().expect("user is authenticated");

    for device_id in db.users.all_device_ids(sender_user) {
        if let Ok(device_id) = device_id {
            db.users.remove_device(&sender_user, &device_id)?;
        }
    }

    Ok(logout_all::Response::new().into())
}