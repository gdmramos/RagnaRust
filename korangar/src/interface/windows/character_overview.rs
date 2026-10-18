use korangar_interface::window::{CustomWindow, Window};
use rust_state::{Path, PathExt, Selector};

use crate::graphics::Color;
use crate::input::InputEvent;
use crate::interface::windows::WindowClass;
use crate::loaders::OverflowBehavior;
use crate::state::localization::LocalizationPathExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};
use crate::world::{CommonPathExt, Player, PlayerPathExt};

/// Formats a number with thousands separators, e.g. `1234567` becomes
/// `1,234,567`.
fn format_number(value: u64) -> String {
    let digits = value.to_string();
    let length = digits.len();
    let mut result = String::with_capacity(length + length / 3);

    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (length - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(character);
    }

    result
}

pub struct CharacterOverviewWindow<A, B> {
    player_name_path: A,
    player_path: B,
}

impl<A, B> CharacterOverviewWindow<A, B> {
    pub fn new(player_name_path: A, player_path: B) -> Self {
        Self {
            player_name_path,
            player_path,
        }
    }
}

impl<A, B> CustomWindow<ClientState> for CharacterOverviewWindow<A, B>
where
    A: Path<ClientState, String>,
    B: Path<ClientState, Player>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::CharacterOverview)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        /// Renders `current / maximum`, e.g. `120 / 120`.
        fn fraction<C, D>(current_path: C, maximum_path: D) -> impl Selector<ClientState, String>
        where
            C: Path<ClientState, usize>,
            D: Path<ClientState, usize>,
        {
            ComputedSelector::new_default(move |state: &ClientState| {
                let current = *current_path.follow_safe(state);
                let maximum = *maximum_path.follow_safe(state);

                format!("{} / {}", format_number(current as u64), format_number(maximum as u64))
            })
        }

        /// Renders a formatted number with thousands separators.
        fn number<C>(path: C) -> impl Selector<ClientState, String>
        where
            C: Path<ClientState, u64>,
        {
            ComputedSelector::new_default(move |state: &ClientState| format_number(*path.follow_safe(state)))
        }

        /// Renders experience as a percentage together with the raw values,
        /// e.g. `12.3% (1,234 / 10,000)`.
        fn experience<C, D>(current_path: C, maximum_path: D) -> impl Selector<ClientState, String>
        where
            C: Path<ClientState, u64>,
            D: Path<ClientState, u64>,
        {
            ComputedSelector::new_default(move |state: &ClientState| {
                let current = *current_path.follow_safe(state);
                let maximum = *maximum_path.follow_safe(state);

                let percentage = match maximum {
                    0 => 0.0,
                    maximum => current as f64 / maximum as f64 * 100.0,
                };

                format!("{percentage:.1}% ({} / {})", format_number(current), format_number(maximum))
            })
        }

        macro_rules! info_row {
            ($label:expr, $value:expr, $row_color:expr) => {
                split! {
                    children: (
                        text! {
                            text: $label,
                            overflow_behavior: OverflowBehavior::Shrink,
                        },
                        text! {
                            text: $value,
                            color: $row_color,
                            horizontal_alignment: HorizontalAlignment::Right { offset: 0.0, border: 3.0 },
                            overflow_behavior: OverflowBehavior::Shrink,
                        },
                    ),
                }
            };
        }

        let name_color = Color::rgb_u8(255, 144, 13);
        let level_color = Color::rgb_u8(13, 231, 255);
        let health_color = Color::rgb_u8(240, 96, 96);
        let spell_color = Color::rgb_u8(96, 168, 255);
        let money_color = Color::rgb_u8(255, 215, 0);
        let base_experience_color = Color::rgb_u8(150, 215, 150);
        let job_experience_color = Color::rgb_u8(230, 200, 120);

        window! {
            title: client_state().localization().character_overview_window_title(),
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            minimum_width: 300.0,
            maximum_width: 300.0,
            elements: (
                fragment! {
                    gaps: 4.0,
                    children: (
                        info_row!(
                            client_state().localization().name_text(),
                            self.player_name_path,
                            name_color
                        ),
                        info_row!(
                            client_state().localization().base_level_text(),
                            PartialEqDisplaySelector::new(self.player_path.base_level()),
                            level_color
                        ),
                        info_row!(
                            client_state().localization().job_level_text(),
                            PartialEqDisplaySelector::new(self.player_path.job_level()),
                            level_color
                        ),
                        info_row!(
                            client_state().localization().health_points_text(),
                            fraction(
                                self.player_path.common().health_points(),
                                self.player_path.common().maximum_health_points()
                            ),
                            health_color
                        ),
                        info_row!(
                            client_state().localization().spell_points_text(),
                            fraction(self.player_path.spell_points(), self.player_path.maximum_spell_points()),
                            spell_color
                        ),
                        info_row!(
                            client_state().localization().money_text(),
                            number(self.player_path.money()),
                            money_color
                        ),
                        info_row!(
                            client_state().localization().base_experience_text(),
                            experience(self.player_path.base_experience(), self.player_path.next_base_experience()),
                            base_experience_color
                        ),
                        info_row!(
                            client_state().localization().job_experience_text(),
                            experience(self.player_path.job_experience(), self.player_path.next_job_experience()),
                            job_experience_color
                        ),
                    ),
                },
                collapsible! {
                    text: client_state().localization().menu_section_text(),
                    initially_expanded: true,
                    children: (
                        button! {
                            text: client_state().localization().inventory_button_text(),
                            event: InputEvent::ToggleInventoryWindow,
                        },
                        button! {
                            text: client_state().localization().equipment_button_text(),
                            event: InputEvent::ToggleEquipmentWindow,
                        },
                        button! {
                            text: client_state().localization().skill_tree_button_text(),
                            event: InputEvent::ToggleSkillTreeWindow,
                        },
                        button! {
                            text: client_state().localization().stats_button_text(),
                            event: InputEvent::ToggleStatsWindow,
                        },
                        button! {
                            text: client_state().localization().friend_list_button_text(),
                            event: InputEvent::ToggleFriendListWindow,
                        },
                        button! {
                            text: client_state().localization().menu_button_text(),
                            event: InputEvent::ToggleMenuWindow,
                        },
                    ),
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_number;

    #[test]
    fn format_number_inserts_thousands_separators() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(42), "42");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1_000), "1,000");
        assert_eq!(format_number(1_234_567), "1,234,567");
    }
}
