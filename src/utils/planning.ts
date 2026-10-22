import type { CommitmentPreview, GameState, QuestEntryPlan } from '../types/game';
import { getLabel } from '../types/game';
import { calendarDate, formatCalendarDay } from './calendar';

export function commitmentMessage(state: GameState, preview: CommitmentPreview): string {
  const text = (key: string, fallback: string) => getLabel(state.catalog, key, fallback);
  const date = (day: number) => formatCalendarDay(day, state.days_per_year, state.catalog.labels.values);
  const resourceName = (id: string) => {
    const actual = id === 'budget' ? state.catalog.resource_roles?.currency || id
      : id === 'stamina' ? state.catalog.resource_roles?.recovery || id : id;
    return state.catalog.player_characteristics.find((entry) => entry.id === actual)?.name || actual;
  };
  const lines = [
    text('commitment_intro', 'Review these costs and commitments before continuing. Time is paused; you can resume it after deciding.'),
    `${text('commitment_upfront', 'Immediate costs')}: ${preview.upfront_currency} ${resourceName(state.catalog.resource_roles?.currency || '')}; ${preview.upfront_recovery} ${resourceName(state.catalog.resource_roles?.recovery || '')}.`,
  ];
  if (preview.conditional_payout) lines.push(`${text('commitment_payout', 'Payout only if accepted/successful')}: ${preview.conditional_payout}.`);
  for (const obligation of preview.obligations) {
    const weekdays = obligation.due_days.split(';').filter(Boolean).map((day) =>
      /^[1-7]$/.test(day.trim()) ? calendarDate(Number(day), 7, 'yeardays_only').weekdayName : day,
    ).join(', ');
    lines.push(
      `${resourceName(obligation.resource)}: ${obligation.amount} / ${obligation.interval} ${obligation.interval_unit}`
      + (weekdays ? ` (${text('commitment_due_days', 'due weekdays')}: ${weekdays})` : '')
      + (obligation.first_due_day !== null ? `; ${text('commitment_first_due', 'first due')}: ${date(obligation.first_due_day)}` : `; ${text('commitment_no_due_date', 'No matching due date found; check the configured schedule.')}`)
      + (obligation.max_payments ? `; ${obligation.max_payments} ${text('commitment_payments', 'payments')}, ${text('commitment_total', 'total')}: ${obligation.amount * obligation.max_payments}` : `; ${text('commitment_ongoing', 'ongoing until ended')}`),
    );
    if (obligation.fault_limit) {
      lines.push(`${text('commitment_fault_limit', 'Consequence after this many missed obligations')}: ${obligation.fault_limit}. ${obligation.consequence_message || obligation.fault_consequence}`);
    }
  }
  if (preview.accepted_forecast) {
    lines.push('', text('commitment_projection', 'Conditional projection if accepted, with no further actions or new random incidents:'));
    for (const resource of preview.accepted_forecast.resources) {
      lines.push(`${resource.name}: ${text('forecast_end', 'projected end')} ${resource.projected.toFixed(2)}; ${text('forecast_minimum', 'lowest daily balance')} ${resource.minimum.toFixed(2)}.`);
    }
    if (preview.accepted_forecast.terminal_day !== null) {
      lines.push(`${text('forecast_terminal', 'Projected terminal state on')} ${date(preview.accepted_forecast.terminal_day)}.`);
    }
  }
  return lines.join('\n\n');
}

export function fillQuestPlan(document: Document, plan: QuestEntryPlan, state: GameState): void {
  const text = (key: string, fallback: string) => getLabel(state.catalog, key, fallback);
  const currency = text('currency_symbol', '');
  const money = (amount: number) => `${currency}${amount.toLocaleString(undefined, { maximumFractionDigits: 2 })}`;
  for (const slot of document.querySelectorAll<HTMLElement>('[data-engine-slot="quest-plan"]')) {
    if (slot.dataset.questId && slot.dataset.questId !== plan.quest_id) continue;
    slot.replaceChildren();
    const add = (parent: Element, tag: string, content: string) => {
      const element = document.createElement(tag);
      element.textContent = content;
      parent.append(element);
      return element;
    };
    add(slot, 'h2', text('quest_planning_title', 'How do I enter?'));
    add(slot, 'p', text('quest_planning_assumptions', 'Estimates use current prerequisites and purchase prices. They exclude future running costs, rentals, negotiated support and random outcomes. Recheck eligibility on the event date.'));
    add(slot, 'p', plan.eligibility.joined
      ? text('quest_planning_joined', 'Already enrolled')
      : `${text('quest_planning_join_fee', 'Enrolment fee')}: ${money(plan.join_fee)}. ${plan.eligibility.can_join
        ? text('quest_planning_can_join', 'You can enrol now; event readiness is separate.')
        : plan.eligibility.reason}`);
    if (!plan.events.length) add(slot, 'p', text('quest_planning_no_events', 'No scheduled events are configured.'));
    for (const event of plan.events) {
      const details = document.createElement('details');
      slot.append(details);
      details.open = event === plan.events[0];
      add(details, 'summary', `${event.name} — ${formatCalendarDay(event.next_day, state.days_per_year, state.catalog.labels.values)}`);
      for (const preparation of event.preparations) {
        const section = document.createElement('section');
        section.className = 'planning-option';
        details.append(section);
        add(section, 'h3', preparation.selection_name);
        add(section, 'p', `${text('quest_planning_estimate', 'Estimated preparation + enrolment + this entry')}: ${money(preparation.total_cost)}`
          + ` (${text('quest_planning_entry_fee', 'event entry')}: ${money(preparation.entry_fee)}).`);
        if (preparation.shortfall > 0) add(section, 'p', `${text('quest_planning_shortfall', 'Additional funds needed')}: ${money(preparation.shortfall)}`);
        const list = document.createElement('ul');
        section.append(list);
        for (const item of preparation.purchases) add(list, 'li', `${item.name}: ${money(item.cost)}`);
        if (!preparation.purchases.length) add(section, 'p', text('quest_planning_no_purchases', 'No additional purchases in this option.'));
        for (const blocker of preparation.blockers) add(section, 'p', `${text('quest_planning_blocker', 'Unresolved requirement')}: ${blocker}`);
        if (preparation.ready_after_preparation) add(section, 'p', text('quest_planning_ready', 'Entry checks pass in this hypothetical setup; funding and future conditions still need to be met.'));
      }
    }
  }
}
