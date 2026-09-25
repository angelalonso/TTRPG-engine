#!/usr/bin/env node

import fs from 'node:fs';

const path = process.argv[2] || 'playtest.out';
if (!fs.existsSync(path)) {
  console.error(`playtest analysis: file not found: ${path}`);
  process.exit(1);
}
const text = fs.readFileSync(path, 'utf8');
const lines = text.split(/\r?\n/);

const outcomes = new Map();
const daysByOutcome = new Map();
const decisions = new Map();
const actions = new Map();
const events = new Map();
let overrideErrors = 0;
let encounterErrors = 0;
let failedDecisions = 0;
let parsedOutcomes = 0;

const increment = (map, key, amount = 1) => map.set(key, (map.get(key) || 0) + amount);
const sortedEntries = (map) => [...map.entries()].sort((left, right) => right[1] - left[1]);

for (const line of lines) {
  if (line.includes('override_error=')) overrideErrors += 1;
  if (line.includes('encounter_error=')) encounterErrors += 1;
  if (line.includes('failed:')) failedDecisions += 1;

  const outcome = line.match(/seed=(\d+) outcome=([a-z_]+) days=(\d+)/);
  if (outcome) {
    const [, , name, dayText] = outcome;
    increment(outcomes, name);
    parsedOutcomes += 1;
    if (!daysByOutcome.has(name)) daysByOutcome.set(name, []);
    daysByOutcome.get(name).push(Number(dayText));
  }

  const decision = line.match(
    /decision=(Action|Event|Purchase|JoinQuest)\("([^"]+)"\)|decision=(Wait)\b/,
  );
  if (decision) {
    const kind = decision[1] || decision[3];
    const name = decision[2] || kind;
    increment(decisions, name);
    if (kind === 'Action') increment(actions, name);
    if (kind === 'Event') increment(events, name.split(',')[0].trim());
  }

  const action = line.match(/\baction=([^ ]+) success=/);
  if (action && !decision) increment(actions, action[1]);

  const event = line.match(/\bevent=([^ ]+) result=/);
  if (event && !decision) increment(events, event[1]);
}

if (parsedOutcomes === 0) {
  console.error(`playtest analysis: no run outcomes found in ${path}`);
  process.exit(1);
}

const totalRuns = [...outcomes.values()].reduce((sum, value) => sum + value, 0);
const percentile = (values, fraction) => {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.round((sorted.length - 1) * fraction)];
};
const formatTop = (entries, limit = 12) =>
  entries.slice(0, limit).map(([name, count]) => `  ${count.toString().padStart(6)} ${name}`).join('\n')
  || '  (none)';

const deathDays = [...daysByOutcome.entries()]
  .filter(([name]) => name.startsWith('dead'))
  .flatMap(([, values]) => values);

console.log(`file=${path}`);
console.log(`runs=${totalRuns}`);
console.log('\noutcomes:');
console.log(formatTop(sortedEntries(outcomes), 20));
if (deathDays.length > 0) {
  console.log(
    `\ndeath days: min=${Math.min(...deathDays)} p10=${percentile(deathDays, 0.1)} `
    + `p50=${percentile(deathDays, 0.5)} p90=${percentile(deathDays, 0.9)} max=${Math.max(...deathDays)}`,
  );
}
console.log('\ntop decisions:');
console.log(formatTop(sortedEntries(decisions)));
console.log('\ntop applied actions:');
console.log(formatTop(sortedEntries(actions)));
console.log('\ntop entered events:');
console.log(formatTop(sortedEntries(events)));
console.log(
  `\nerrors: override=${overrideErrors} encounter=${encounterErrors} failed_decisions=${failedDecisions}`,
);
