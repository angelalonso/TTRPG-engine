use crate::{EventHistory, GameState, OwnedObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactError {
    MissingCharacteristic(String),
    MissingObjectDefinition(String),
    MissingEvent(String),
    MissingQuest(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectQuery {
    pub definition_id: Option<String>,
    pub object_type: Option<String>,
    pub include_loaned: bool,
    pub include_unusable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateFact {
    Characteristic {
        id: String,
        value: f64,
    },
    ObjectCount {
        query: ObjectQuery,
        count: usize,
    },
    EventCompleted {
        event_id: String,
        success: bool,
    },
    QuestJoined {
        quest_id: String,
        joined: bool,
    },
    ActiveEventCount {
        event_id: Option<String>,
        count: usize,
    },
    CalendarDay(u32),
    AgeDays(u32),
}

pub fn characteristic(game: &GameState, id: &str) -> Result<f64, FactError> {
    if !game
        .catalog
        .player_characteristics
        .iter()
        .any(|entry| entry.id == id)
    {
        return Err(FactError::MissingCharacteristic(id.to_string()));
    }
    Ok(game
        .player
        .characteristics
        .get(id)
        .copied()
        .unwrap_or_default())
}

pub fn object_count(game: &GameState, query: &ObjectQuery) -> Result<usize, FactError> {
    if let Some(definition_id) = &query.definition_id {
        if !game
            .catalog
            .objects
            .iter()
            .any(|object| object.id == *definition_id)
        {
            return Err(FactError::MissingObjectDefinition(definition_id.clone()));
        }
    }
    let mut count = 0;
    for object in &game.player.inventory {
        if matches_object(game, object, query) {
            count += 1;
        }
    }
    Ok(count)
}

pub fn event_completed(
    game: &GameState,
    event_id: &str,
    success: Option<bool>,
) -> Result<bool, FactError> {
    if !game.catalog.events.iter().any(|event| event.id == event_id) {
        return Err(FactError::MissingEvent(event_id.to_string()));
    }
    Ok(game.player_history().iter().any(|history| {
        history.event_id == event_id
            && success.is_none_or(|expected| {
                history
                    .result
                    .eq_ignore_ascii_case(if expected { "success" } else { "failure" })
            })
    }))
}

pub fn quest_joined(game: &GameState, quest_id: &str) -> Result<bool, FactError> {
    if !game.catalog.quests.iter().any(|quest| quest.id == quest_id) {
        return Err(FactError::MissingQuest(quest_id.to_string()));
    }
    Ok(game
        .quest_memberships
        .iter()
        .any(|membership| membership.quest_id == quest_id))
}

pub fn active_event_count(game: &GameState, event_id: Option<&str>) -> usize {
    game.player
        .active_events
        .iter()
        .filter(|event| event_id.is_none_or(|expected| event.event_id == expected))
        .count()
}

pub fn evaluate_fact(game: &GameState, fact: StateFact) -> Result<bool, FactError> {
    match fact {
        StateFact::Characteristic { id, value } => Ok(characteristic(game, &id)? == value),
        StateFact::ObjectCount { query, count } => Ok(object_count(game, &query)? == count),
        StateFact::EventCompleted { event_id, success } => {
            event_completed(game, &event_id, Some(success))
        }
        StateFact::QuestJoined { quest_id, joined } => Ok(quest_joined(game, &quest_id)? == joined),
        StateFact::ActiveEventCount { event_id, count } => {
            Ok(active_event_count(game, event_id.as_deref()) == count)
        }
        StateFact::CalendarDay(day) => Ok(game.current_day == day),
        StateFact::AgeDays(days) => Ok(game.player.age_days == days),
    }
}

fn matches_object(game: &GameState, object: &OwnedObject, query: &ObjectQuery) -> bool {
    if let Some(definition_id) = &query.definition_id {
        if object.definition_id != *definition_id && object.id != *definition_id {
            return false;
        }
    }
    if let Some(object_type) = &query.object_type {
        let Some(definition) =
            game.catalog.objects.iter().find(|definition| {
                definition.id == object.definition_id || definition.id == object.id
            })
        else {
            return false;
        };
        if definition.object_type != *object_type {
            return false;
        }
    }
    if !query.include_loaned && object.loaned {
        return false;
    }
    if !query.include_unusable && object.unavailable_until_day > game.current_day {
        return false;
    }
    true
}

trait HistoryAccess {
    fn player_history(&self) -> &[EventHistory];
}

impl HistoryAccess for GameState {
    fn player_history(&self) -> &[EventHistory] {
        &self.event_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_query_defaults_to_usable_non_loaned_instances() {
        let query = ObjectQuery::default();
        assert!(!query.include_loaned);
        assert!(!query.include_unusable);
    }
}
