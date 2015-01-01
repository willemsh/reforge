use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{RingBuf, HashMap};
use std::collections::hash_map::Entry;
use std::sync::Arc;

use battle_state::BattleContext;
use battle_type::BattleType;
use module::ModuleTypeStore;
use net::{ClientId, ServerSlot, SlotInMsg};
use server_battle_state::ServerBattleState;
use ship::{Ship, ShipId};

pub struct BattleScheduler {
    slot: ServerSlot,
    ffa_waiting: HashMap<u8, Vec<ClientId>>,
    mod_store: Arc<ModuleTypeStore>,
}

impl BattleScheduler {
    pub fn new(slot: ServerSlot, mod_store: Arc<ModuleTypeStore>) -> BattleScheduler {
        BattleScheduler {
            slot: slot,
            ffa_waiting: HashMap::new(),
            mod_store: mod_store,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.slot.receive() {
                SlotInMsg::Joined(client_id) => {
                    println!("Client {} joined the scheduler", client_id);
                },
                SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                    let battle_type: BattleType = packet.read().ok().expect("Battle scheduler failed to read battle type from client.");
                    match battle_type {
                        BattleType::FreeForAll{num_players: num_players} => {
                            match self.ffa_waiting.entry(num_players) {
                                Entry::Vacant(entry) => { entry.set(vec![client_id]); },
                                Entry::Occupied(mut entry) => {
                                    let waiting = entry.get_mut();
                                    
                                    // Add the client to the waiting list
                                    waiting.push(client_id);
                                    
                                    // Chech if we're ready to schedule
                                    if waiting.len() == num_players as uint {
                                        let new_slot = self.slot.create_slot_and_transfer_clients(waiting);
                                        schedule_ffa(new_slot, self.mod_store.clone(), waiting.clone());
                                        waiting.clear();
                                    }
                                },
                            }
                        },
                        BattleType::Ai => {
                            let new_slot = self.slot.create_slot_and_transfer_clients(&vec![client_id]);
                            schedule_ai(new_slot, self.mod_store.clone(), client_id);
                        },
                    }
                },
                _ => {}
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Scheduling functions

fn schedule_ai(new_slot: ServerSlot, mod_store: Arc<ModuleTypeStore>, client_id: ClientId) {
    spawn(move || {
        // Create ships
        let mut ship1 = Ship::generate(mod_store.deref(), client_id as ShipId);
        ship1.client_id = Some(client_id);
        
        // TODO: come up with better way to generate AI ship IDs
        let mut ship2 = Ship::generate(mod_store.deref(), (100000000 - client_id) as ShipId);
        ship2.client_id = None;
    
        // Map clients to their ships
        let mut ships = vec!();
        ships.push(Rc::new(RefCell::new(ship1)));
        ships.push(Rc::new(RefCell::new(ship2)));
    
        let mut battle_state = ServerBattleState::new(new_slot, BattleContext::new(ships));
        battle_state.run();
    });
}

fn schedule_ffa(new_slot: ServerSlot, mod_store: Arc<ModuleTypeStore>, clients: Vec<ClientId>) {
    spawn(move || {
        let mut ships = vec!();
        for client_id in clients.iter() {
            // Create player's ship
            let mut ship = Ship::generate(mod_store.deref(), *client_id as ShipId);
            ship.client_id = Some(*client_id);
            
            // Add to the list of ships
            ships.push(Rc::new(RefCell::new(ship)));
        }
    
        let mut battle_state = ServerBattleState::new(new_slot, BattleContext::new(ships));
        battle_state.run();
    });
}