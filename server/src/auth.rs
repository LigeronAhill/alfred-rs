use tonic::metadata::MetadataValue;
use tonic::{Request, Status};

pub fn check_auth(req: Request<()>) -> anyhow::Result<Request<()>, Status> {
    let bt = std::env::var("BEARER_TOKEN").map_err(|e| Status::internal(format!("{e:?}")))?;
    let token: MetadataValue<_> = format!("Bearer {bt}")
        .parse()
        .map_err(|e| Status::internal(format!("error: {e:?}")))?;

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
