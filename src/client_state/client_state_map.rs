use std::collections::HashMap;

use crate::client_state::{ClientState, ClientStateError};

#[derive(Clone)]
pub struct ClientStateMap {
    map: HashMap<u64, ClientState>,
}

impl ClientStateMap {
    pub fn new() -> Self {
        ClientStateMap {
            map: (HashMap::new()),
        }
    }

    pub fn get(&self, id: &u64) -> Option<&ClientState> {
        self.map.get(id)
    }

    pub fn contains_key(&self, id: &u64) -> bool {
        self.map.contains_key(id)
    }

    pub fn insert(
        &mut self,
        id: &u64,
        client_state: &mut ClientState,
    ) -> Result<(), ClientStateError> {
        if self.map.contains_key(id) {
            return Err(ClientStateError::ReservedClientID);
        }

        self.map.insert(id.clone(), client_state.to_owned());
        Ok(())
    }

    pub fn update(
        &mut self,
        id: &u64,
        client_state: &mut ClientState,
    ) -> Result<(), ClientStateError> {
        match self.map.contains_key(id) {
            true => {
                self.map.insert(id.clone(), client_state.to_owned());
                Ok(())
            }
            false => Err(ClientStateError::NonExistentClientID),
        }
    }

    pub fn remove(&mut self, id: &u64) -> Result<(), ClientStateError> {
        match self.map.contains_key(id) {
            true => {
                self.map.remove(id);
                Ok(())
            }
            false => Err(ClientStateError::NonExistentClientID),
        }
    }
}
