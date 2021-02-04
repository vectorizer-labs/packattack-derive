#[async_trait] impl < R > FromReader < crate :: ERROR, R > for Connect where
Self : Sized, R : Read + std :: marker :: Unpin + std :: marker :: Send
{
    async fn from_reader(reader : & mut R) -> Result < Self, crate :: ERROR >
    {
        let c_flags : ConnectFlags ;
        Ok(Connect(< Protocol > ::
                   from_bytes(& bytes[(() / 8) ..((() + (< Protocol > :: SIZE_IN_BITS)) / 8)])
                   ?, < ProtocolLevel > ::
                   from_bytes(& bytes[(() / 8) ..((() + (< ProtocolLevel > :: SIZE_IN_BITS)) /8)]) ?,
                   {
                       let result : ConnectFlags = < ConnectFlags > ::
                       from_bytes(& bytes[(() / 8) ..((() + (< ConnectFlags > :: SIZE_IN_BITS))/ 8)]) ?;
                       c_flags = result . clone();
                       result
                   }, 
                   < KeepAlive > ::from_bytes(& bytes[(() / 8) ..((() + (< KeepAlive > :: SIZE_IN_BITS)) / 8)])?,
                   < Properties > ::from_bytes(& bytes[(() / 8) ..((() + (< Properties > :: SIZE_IN_BITS)) / 8)])?, 
                   < ClientID > ::from_bytes(& bytes[(() / 8) ..((() + (< ClientID > :: SIZE_IN_BITS)) / 8)])?,
                   {
                       match c_flags . WillFlag
                       {
                           true => Some(< WillProperties > ::from_bytes(& bytes[(() / 8) ..((() +(< WillProperties > ::SIZE_IN_BITS)) / 8)]) ?), 
                           false=> None
                       }
                   },
                   {
                       match c_flags . WillFlag
                       {
                           true => Some(<WillTopic > ::from_bytes(& bytes[(() / 8) ..((() +(< WillTopic > :: SIZE_IN_BITS))/ 8)]) ?), 
                           false => None
                       }
                   },
                   {
                       match c_flags . WillFlag
                       {
                           true => Some(< WillPayload > ::from_bytes(& bytes[(() / 8) ..((() +(< WillPayload > ::SIZE_IN_BITS)) / 8)]) ?), 
                           false => None
                       }
                   },
                   {
                       match c_flags . UserNameFlag
                       {
                           true => Some(< Username > ::from_bytes(& bytes[(() / 8) ..((() + (< Username > :: SIZE_IN_BITS))/ 8)]) ?), 
                           false => None
                       }
                   },
                   {
                       match c_flags . PasswordFlag
                       {
                           true =>
                           Some(< Password > ::
                                from_bytes(& bytes
                                           [(() / 8) ..
                                            ((() +
                                              (< Password > :: SIZE_IN_BITS))
                                             / 8)]) ?), false => None
                       }
                   }))
    }
}