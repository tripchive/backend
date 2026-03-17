pub mod apple;
pub mod github;
pub mod google;

use oauth2::{EndpointNotSet, EndpointSet, basic::BasicClient};

pub type OAuthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;
