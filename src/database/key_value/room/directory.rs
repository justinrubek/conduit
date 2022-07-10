impl service::room::directory::Data for KeyValueDatabase {
    fn set_public(&self, room_id: &RoomId) -> Result<()> {
        self.publicroomids.insert(room_id.as_bytes(), &[])?;
    }

    fn set_not_public(&self, room_id: &RoomId) -> Result<()> {
        self.publicroomids.remove(room_id.as_bytes())?;
    }

    fn is_public_room(&self, room_id: &RoomId) -> Result<bool> {
        Ok(self.publicroomids.get(room_id.as_bytes())?.is_some())
    }

    fn public_rooms(&self) -> impl Iterator<Item = Result<Box<RoomId>>> + '_ {
        self.publicroomids.iter().map(|(bytes, _)| {
            RoomId::parse(
                utils::string_from_bytes(&bytes).map_err(|_| {
                    Error::bad_database("Room ID in publicroomids is invalid unicode.")
                })?,
            )
            .map_err(|_| Error::bad_database("Room ID in publicroomids is invalid."))
        })
    }
}
