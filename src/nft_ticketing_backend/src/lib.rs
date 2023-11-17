// use ic_cdk::export::candid::{CandidType, Principal};
use candid::{CandidType, Principal};
use ic_cdk::{query, update};
// use candid::Deserialize;

use once_cell::sync::Lazy;
use std::clone::Clone;
use std::collections::HashMap;
#[derive(CandidType, Clone)]
struct Event {
    id: String,
    name: String,
    date: String,
    location: String,
    max_seats: u32,
    nft_id: Option<String>,
}

#[derive(CandidType, Clone)]
struct Ticket {
    id: String,
    seat_number: String,
    event_id: String,
    owner: Principal,
}

#[derive(CandidType)]
struct NFTMetadata {
    token_id: String,
    owner: Principal,
    metadata: DIP721Metadata,
}

#[derive(CandidType)]
struct DIP721Metadata {
    name: String,               // Name of the NFT (e.g., "Event Ticket")
    description: String,        // Description of the NFT
    image: String,              // URL or data URI for an image
    attributes: Vec<Attribute>, // Additional attributes (e.g., event details)
}

#[derive(CandidType)]
struct Attribute {
    trait_type: String, // Type of the attribute (e.g., "Event Name")
    value: String,      // Value of the attribute (e.g., "Concert XYZ")
}

#[derive(CandidType)]
struct Metadata {
    name: String,
    description: String,
    image: String,
    attributes: Vec<Attribute>,
}
enum Error {
    NotFound,
    NotAuthorized,
    InvalidInput,
}

// Global state for managing events and tickets
static mut EVENTS: Lazy<HashMap<String, Event>> = Lazy::new(|| HashMap::new());
static mut TICKETS: Lazy<HashMap<String, Ticket>> = Lazy::new(|| HashMap::new());
static mut NFT_METADATA: Lazy<HashMap<String, NFTMetadata>> = Lazy::new(|| HashMap::new());

#[ic_cdk::update]
fn create_event(
    name: String,
    date: String,
    location: String,
    num_seats: u32,
    id: String,
) -> Result<Event, String> {
    unsafe {
        if EVENTS.contains_key(&id) {
            return Err("Event with this ID already exists".to_string());
        }

        let event = Event {
            id: id.clone(),
            name,
            date,
            location,
            max_seats: num_seats,
            nft_id: None, // This can be updated later when NFTs are minted
        };

        EVENTS.insert(id.clone(), event.clone());

        Ok(event)
    }
}

#[ic_cdk::update]
fn mint_ticket(event_id: String, seat_number: u32, owner: Principal) -> Result<Ticket, String> {
    unsafe {
        let event = match EVENTS.get(&event_id) {
            Some(e) => e,
            None => return Err("Event not found".to_string()),
        };

        if seat_number > event.max_seats {
            return Err("Seat number exceeds the maximum seats available".to_string());
        }

        let ticket_id = format!("{}_{}", event_id, seat_number); // Unique ID for the ticket

        if TICKETS.contains_key(&ticket_id) {
            return Err("This seat is already taken".to_string());
        }

        // Mint the NFT here following DIP-721 standard
        let nft_metadata = NFTMetadata {
            token_id: ticket_id.clone(),
            owner: owner.clone(),
            metadata: DIP721Metadata {
                name: "Event Ticket".to_string(),
                description: format!("Ticket for {} at seat {}", event.name, seat_number),
                image: "image_url_or_data_uri".to_string(), // Replace with actual image URL or data URI
                attributes: vec![
                    Attribute {
                        trait_type: "Event Name".to_string(),
                        value: event.name.clone(),
                    },
                    // Add other event-related attributes here
                ],
            },
        };

        NFT_METADATA.insert(ticket_id.clone(), nft_metadata);

        let ticket = Ticket {
            id: ticket_id.clone(),
            seat_number: seat_number.to_string(),
            event_id: event_id.clone(),
            owner: owner.clone(),
        };

        TICKETS.insert(ticket_id, ticket.clone());

        Ok(ticket)
    }
}
#[ic_cdk::update]
fn transfer_ticket(
    ticket_id: String,
    new_owner: Principal,
    owner: Principal,
) -> Result<(), String> {
    unsafe {
        // Check if the ticket exists
        let ticket = match TICKETS.get_mut(&ticket_id) {
            Some(t) => t,
            None => return Err("Ticket not found".to_string()),
        };

        // Check if the caller is the current owner of the ticket
        if ticket.owner != owner {
            return Err("Only the ticket owner can transfer it".to_string());
        }

        // Update the ticket's owner
        ticket.owner = new_owner;

        Ok(())
    }
}

#[ic_cdk::query]
fn check_ticket_owner(ticket_id: u64) -> Option<Principal> {
    // Access the global TICKETS HashMap in a safe way
    unsafe {
        // Check if the ticket with the given ID exists
        TICKETS
            .get(&ticket_id.to_string())
            .map(|ticket| ticket.owner)
    }
}

#[ic_cdk::query]
fn get_event(event_id: String) -> Option<Event> {
    // Access the global EVENTS HashMap in a safe way
    unsafe {
        // Check if the event with the given ID exists
        EVENTS.get(&event_id).cloned()
    }
}
