use super::esphome_entities::EsphomeEntities;
use super::home_assistant_entities::HomeAssistantEntities;

#[derive(Debug, Clone)]
pub enum ModelEntities {
    Payload,
    Esphome(EsphomeEntities),
    HomeAssistant(HomeAssistantEntities),
}
