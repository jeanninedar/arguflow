use super::message_handler::get_topic_string;
use crate::{
    data::models::{Pool, Topic},
    errors::{DefaultError, ServiceError},
    handlers::auth_handler::LoggedUser,
    operators::topic_operator::{
        create_topic_query, delete_topic_query, get_all_topics_for_user_query,
        get_topic_for_user_query, update_topic_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateTopicData {
    pub resolution: String,
    pub normal_chat: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/topic",
    context_path = "/api",
    tag = "topic",
    request_body(content = CreateTopicData, description = "JSON request payload to create chat topic", content_type = "application/json"),
    responses(
        (status = 200, description = "The JSON response payload containing the created topic", body = [Topic]),
        (status = 400, description = "Topic resolution empty or a service error", body = [DefaultError]),
    )
)]
pub async fn create_topic(
    data: web::Json<CreateTopicData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let resolution = data_inner.resolution;
    let normal_chat = data_inner.normal_chat;

    if resolution.is_empty() {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Resolution must not be empty",
        }));
    }

    let topic_resolution = get_topic_string(resolution)
        .await
        .map_err(|e| ServiceError::BadRequest(format!("Error getting topic string: {}", e)))?;

    let new_topic = Topic::from_details(topic_resolution, user.id, normal_chat);
    let new_topic1 = new_topic.clone();

    let create_topic_result = web::block(move || create_topic_query(new_topic, &pool)).await?;

    match create_topic_result {
        Ok(()) => Ok(HttpResponse::Ok().json(new_topic1)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteTopicData {
    pub topic_id: uuid::Uuid,
}

#[utoipa::path(
    delete,
    path = "/topic",
    context_path = "/api",
    tag = "topic",
    request_body(content = DeleteTopicData, description = "JSON request payload to delete a chat topic", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the topic was deleted"),
        (status = 400, description = "Service error relating to topic deletion", body = [DefaultError]),
    )
)]
pub async fn delete_topic(
    data: web::Json<DeleteTopicData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let topic_id = data_inner.topic_id;
    let pool_inner = pool.clone();

    let user_topic =
        web::block(move || get_topic_for_user_query(user.id, topic_id, &pool_inner)).await?;

    match user_topic {
        Ok(topic) => {
            let delete_topic_result =
                web::block(move || delete_topic_query(topic.id, &pool)).await?;

            match delete_topic_result {
                Ok(()) => Ok(HttpResponse::NoContent().finish()),
                Err(e) => Ok(HttpResponse::BadRequest().json(e)),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateTopicData {
    pub topic_id: uuid::Uuid,
    pub resolution: String,
    pub side: bool,
}

#[utoipa::path(
    put,
    path = "/topic",
    context_path = "/api",
    tag = "topic",
    request_body(content = UpdateTopicData, description = "JSON request payload to update a chat topic", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the topic was updated"),
        (status = 400, description = "Service error relating to topic update", body = [DefaultError]),
    )
)]
pub async fn update_topic(
    data: web::Json<UpdateTopicData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let topic_id = data_inner.topic_id;
    let resolution = data_inner.resolution;
    let side = data_inner.side;
    let pool_inner = pool.clone();

    if resolution.is_empty() {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Resolution must not be empty",
        }));
    }

    let user_topic =
        web::block(move || get_topic_for_user_query(user.id, topic_id, &pool_inner)).await?;

    match user_topic {
        Ok(topic) => {
            let update_topic_result =
                web::block(move || update_topic_query(topic.id, resolution, side, &pool)).await?;

            match update_topic_result {
                Ok(()) => Ok(HttpResponse::NoContent().finish()),
                Err(e) => Ok(HttpResponse::BadRequest().json(e)),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[utoipa::path(
    get,
    path = "/topic",
    context_path = "/api",
    tag = "topic",
    responses(
        (status = 200, description = "All topics belonging to a given user", body = [Vec<Topic>]),
        (status = 400, description = "Service error relating to topic get", body = [DefaultError]),
    )
)]
pub async fn get_all_topics(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topics = web::block(move || get_all_topics_for_user_query(user.id, &pool)).await?;

    match topics {
        Ok(topics) => Ok(HttpResponse::Ok().json(topics)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
