mod routes_model_router;

pub use routes_model_router::*;

#[cfg(test)]
#[path = "test_model_router_crud.rs"]
mod test_model_router_crud;
