use std::time::SystemTime;

use phonenumber::country::Id::SY;
use ::serde::{Deserialize, Serialize};


#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub enum PostType {
    #[default] ISO,
    OSI,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub enum PostState {
    #[default] Draft,
    Posted,
    Accepted,
    Expired,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub enum TimeType {
    ServiceNow,
    ServiceFuture,
    #[default] ItemPermanant,
    ItemLoan,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub struct Post {
    pub uuid: String,
    pub title: String,
    pub iso_or_osi: PostType,
    pub state: PostState,
    pub location_string: String,
    
    pub time_posted: u64,
    pub time_expires: u64,
    pub time_accepted: Option<u64>,

    user_owner: String,
    user_acceptor: Option<String>,

    karma_diff: i32,

    views: u64,

    time_type: TimeType,
    tags: Vec<String>,
}

impl Post {
    pub fn get_owner(&self) -> String {
        self.user_owner.clone()
    }

    pub fn new(title: String, post_type: PostType, owner_uuid: String, time_type: TimeType, tags: Vec<String>, location_string: String) -> Post {
        let mut post = Post::default();

        post.title = title;
        post.uuid = uuid::Uuid::new_v4().to_string();
        post.iso_or_osi = post_type;
        post.state = PostState::Draft;
        // now
        post.time_posted = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
        // in 24 hours
        post.time_expires = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() + 86400;
        post.time_accepted = None;
        post.user_owner = owner_uuid;
        post.user_acceptor = None;
        post.karma_diff = -1;
        post.views = 0;
        post.time_type = time_type;
        post.tags = tags;
        post.location_string = location_string;

        post
    }

    pub fn claim(&mut self, user_uuid: String) {
        self.user_acceptor = Some(user_uuid);
        self.state = PostState::Accepted;
        self.time_accepted = Some(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs());
    }
}