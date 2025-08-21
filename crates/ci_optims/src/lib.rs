// Dummy crate for pre-compiling dependencies in CI builds
// This crate exists solely to trigger compilation of heavy dependencies
// in a separate Docker layer that can be cached effectively.

#![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]

// Import heavy dependencies to trigger their compilation
use aes_gcm;
use anyhow;
use async_openai;
use async_trait;
use axum;
use base64;
use bytes;
use chrono;
use clap;
use cookie;
use derive_builder;
use dialoguer;
use dirs;
use fluent;
use fs_extra;
use futures;
use hf_hub;
use http;
use hyper;
use jsonwebtoken;
use oauth2;
use regex;
use reqwest;
use serde;
use serde_json;
use sqlx;
use tokio;
use tower;
use tower_http;
use tower_sessions;
use tracing;
use utoipa;
use uuid;

pub fn dummy_function() {
  // This function does nothing but ensures all dependencies are linked
  println!("CI optimizations crate loaded");
}
