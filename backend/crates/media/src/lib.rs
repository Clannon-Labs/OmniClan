
mod routes;
mod upload;
mod utils;
// mod constants;

pub use routes::media_routes;

// Meaning of extensions of files
/*  .part  = still receiving or incomplete
    .ready = body completed, waiting to commit
    .final = committed
*/
