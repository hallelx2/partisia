use std::collections::HashMap;
use num_traits::FromPrimitive;

// Traits for voter eligibility and verification
trait Eligible {
    fn is_eligible(&self) -> bool;
    fn verify_identity(&self, id: &str) -> bool;
}

// Concrete implementation for eligible voters
struct RegisteredVoter {
    name: String,
    age: u32,
    location: String,
}

impl Eligible for RegisteredVoter {
    fn is_eligible(&self) -> bool {
        self.age >= 18 && self.location == "EligibleRegion"
    }

    fn verify_identity(&self, id: &str) -> bool {
        // Implement secure identity verification (e.g., using cryptographic signatures)
        id == "valid_id"
    }
}

// Structure for candidates and their votes
struct Candidate {
    name: String,
    votes: u32,
}

struct VotingContract {
    candidates: Vec<Candidate>,
    eligible_voters: HashMap<String, RegisteredVoter>,
    votes_cast: HashMap<String, u32>, // Voter ID -> Candidate index
}

impl VotingContract {
    fn new(candidates: Vec<Candidate>) -> Self {
        VotingContract {
            candidates,
            eligible_voters: HashMap::new(),
            votes_cast: HashMap::new(),
        }
    }

    fn register_voter(&mut self, voter: RegisteredVoter) {
        if voter.is_eligible() {
            self.eligible_voters.insert(voter.name, voter);
        } else {
            // Handle ineligible voter registration attempt
        }
    }

    fn cast_vote(&mut self, voter_id: &str, candidate_index: usize) -> bool {
        if let Some(voter) = self.eligible_voters.get(voter_id) {
            if voter.verify_identity(voter_id) {
                if !self.votes_cast.contains_key(voter_id) {
                    self.votes_cast.insert(voter_id.to_string(), candidate_index);
                    self.candidates[candidate_index].votes += 1;
                    true
                } else {
                    // Handle attempt to vote twice
                    false
                }
            } else {
                // Handle failed identity verification
                false
            }
        } else {
            // Handle unregistered voter attempting to vote
            false
        }
    }

    fn get_results(&self) -> Vec<String> {
        let mut results: Vec<String> = self.candidates.iter()
            .map(|c| format!("{}: {}", c.name, c.votes))
            .collect();
        results.sort_by(|a, b| b.partial_cmp(a).unwrap()); // Sort in descending order by votes
        results
    }
}


// At this point we begin to initialise the build objects from the structs and 
fn main() {
    let candidates = vec![
        Candidate { name: "Candidate A".to_string(), votes: 0 },
        Candidate { name: "Candidate B".to_string(), votes: 0 },
    ];
    let mut contract = VotingContract::new(candidates);

    // Register eligible voters (replace with actual voter registration process)
    contract.register_voter(RegisteredVoter {
        name: "John Doe".to_string(),
        age: 25,
        location: "EligibleRegion".to_string(),
    });
    contract.register_voter(RegisteredVoter {
        name: "Jane Doe".to_string(),
        age: 30,
        location: "EligibleRegion".to_string(),
    });
