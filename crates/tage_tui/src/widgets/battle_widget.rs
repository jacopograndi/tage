use ratatui::{layout::Alignment, prelude::*, widgets::*};
use tage_core::{actions::attack::ActAttack, prelude::*};

use crate::UnitStatsWidget;

#[derive(Debug, Clone, Copy)]
pub struct BattleWidget<'a> {
    pub board: &'a Board,
    pub blueprints: &'a Blueprints,
    pub atk_pos_moved: IVec2,
    pub def_pos: IVec2,
}

#[derive(Debug, Clone, Default)]
pub struct BattleWidgetState {
    pub temp_board: Option<Board>,
}

impl<'a> StatefulWidget for BattleWidget<'a> {
    type State = BattleWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear::default().render(area, buf);
        Block::bordered()
            .border_type(BorderType::QuadrantOutside)
            .render(area, buf);
        let [area] = Layout::default()
            .constraints([Min(0)])
            .margin(1)
            .areas(area);

        let Some(temp_board) = &state.temp_board else {
            Paragraph::new("Invalid move")
                .alignment(Alignment::Center)
                .render(area, buf);
            return;
        };

        use Constraint::*;
        let [header, body] = Layout::vertical([Length(1), Min(0)]).areas(area);

        Paragraph::new("Battle Overview")
            .alignment(Alignment::Center)
            .render(header, buf);

        let [_, area_atk, _, area_def, _] =
            Layout::horizontal([Fill(1), Length(25), Length(7), Length(25), Fill(1)]).areas(body);

        let (atk_bonus, def_bonus) = match (
            self.board.grid.get_at(&self.atk_pos_moved).get_top_unit(),
            self.board.grid.get_at(&self.def_pos).get_top_unit(),
        ) {
            (Some(atk), Some(def)) => ActAttack {
                this: UnitTarget::new(atk.clone(), self.atk_pos_moved),
                target: UnitTarget::new(def.clone(), self.def_pos),
            }
            .get_attack_bonuses(self.board),
            _ => (Bonus::default(), Bonus::default()),
        };

        for (role, pos, bonus, area_seg, alignment) in [
            (
                "Attacker",
                &self.atk_pos_moved,
                &atk_bonus,
                area_atk,
                Alignment::Right,
            ),
            (
                "Defender",
                &self.def_pos,
                &def_bonus,
                area_def,
                Alignment::Left,
            ),
        ]
        .iter()
        {
            let [_, center, _] =
                Layout::horizontal([Fill(1), Length(UnitStatsWidget::RECT.width + 12), Fill(1)])
                    .areas(*area_seg);
            let [_, body, _] =
                Layout::vertical([Fill(1), Length(UnitStatsWidget::RECT.height + 7), Fill(1)])
                    .areas(center);
            let [header, name, _, stats, _, hp] = Layout::vertical([
                Length(1),
                Length(1),
                Length(1),
                Length(UnitStatsWidget::RECT.height),
                Length(1),
                Length(2),
            ])
            .areas(body);

            Paragraph::new(*role)
                .alignment(*alignment)
                .render(header, buf);

            if let Some(unit) = self.board.grid.get_at(&pos).get_top_unit() {
                let bp = self.blueprints.get_unit(&unit.blueprint_id);
                Paragraph::new(bp.header.name.as_str())
                    .alignment(*alignment)
                    .render(name, buf);

                UnitStatsWidget {
                    stats: &bp.stats,
                    bonus: &Some(bonus),
                    alignment: *alignment,
                }
                .render(stats, buf);

                let outcome = if let Some(damaged) = temp_board
                    .grid
                    .get_at(&pos)
                    .get_unit_by_class(&bp.header.class)
                {
                    format!("{}", damaged.health)
                } else {
                    format!("Dead")
                };
                let health_txt = match alignment {
                    Alignment::Right => format!("Health:\n{} -> {}", unit.health, outcome),
                    _ => format!("Health:\n{} <- {}", outcome, unit.health),
                };
                Paragraph::new(health_txt)
                    .alignment(*alignment)
                    .render(hp, buf);
            } else {
                Paragraph::new("Attacking/Defending a dead unit, eh?")
                    .alignment(Alignment::Center)
                    .render(center, buf);
            }
        }
    }
}
