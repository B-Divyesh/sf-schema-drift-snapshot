import './style.css';

declare const __SDS_BUILD_ID__: string;

type Details = Record<string, unknown>;
type SchemaObject = { kind: string; schema: string; table?: string; name: string; details?: Details };
type Snapshot = { schema_version: number; dialect: string; captured_at: string; objects: SchemaObject[] };
type ReviewChange = { risk: 'high' | 'medium' | 'low'; object: string; title: string; explanation: string };

const PRODUCT = 'schema-drift-snapshot';
const API_BASE = 'https://api.sociobot.in';
const LICENSE_KEY = `sb_license:${PRODUCT}`;
const VERDICT_KEY = `sb_license_verdict:${PRODUCT}`;
const DAY_MS = 86_400_000;
const isDemoRoute = document.body.hasAttribute('data-demo-page');

const expected: Snapshot = {
  schema_version: 1,
  dialect: 'postgresql',
  captured_at: '2026-08-27T12:00:00Z',
  objects: [
    { kind: 'table', schema: 'public', name: 'accounts' },
    { kind: 'column', schema: 'public', table: 'accounts', name: 'email', details: { data_type: 'text', nullable: true } },
    { kind: 'column', schema: 'public', table: 'accounts', name: 'plan', details: { data_type: 'text', nullable: true } },
    { kind: 'index', schema: 'public', table: 'accounts', name: 'accounts_email_idx', details: { unique: true } },
  ],
};

const observed: Snapshot = {
  schema_version: 1,
  dialect: 'postgresql',
  captured_at: '2026-08-27T12:08:00Z',
  objects: [
    { kind: 'table', schema: 'public', name: 'accounts' },
    { kind: 'column', schema: 'public', table: 'accounts', name: 'email', details: { data_type: 'varchar(120)', nullable: false } },
    { kind: 'view', schema: 'public', name: 'active_accounts' },
    { kind: 'index', schema: 'public', table: 'accounts', name: 'accounts_email_idx', details: { unique: false } },
  ],
};

const beforeInput = document.querySelector<HTMLTextAreaElement>('#before-snapshot');
const afterInput = document.querySelector<HTMLTextAreaElement>('#after-snapshot');
const emptyState = document.querySelector<HTMLElement>('#review-empty');
const errorState = document.querySelector<HTMLElement>('#review-error');
const resultState = document.querySelector<HTMLElement>('#review-result');
const summary = document.querySelector<HTMLElement>('#review-summary');
const counts = document.querySelector<HTMLElement>('#review-counts');
const changeList = document.querySelector<HTMLUListElement>('#change-list');

function objectKey(object: SchemaObject): string {
  return [object.kind, object.schema, object.table ?? '', object.name].join(':');
}

function objectLabel(object: SchemaObject): string {
  return [object.schema, object.table, object.name].filter(Boolean).join('.');
}

function parseSnapshot(raw: string, label: string): Snapshot {
  let value: unknown;
  try {
    value = JSON.parse(raw);
  } catch {
    throw new Error(`${label} is not valid JSON. Fix the highlighted snapshot and compare again.`);
  }
  if (!value || typeof value !== 'object') throw new Error(`${label} must be a JSON object.`);
  const snapshot = value as Partial<Snapshot>;
  if (snapshot.schema_version !== 1) throw new Error(`${label} needs schema_version 1.`);
  if (!snapshot.dialect || !Array.isArray(snapshot.objects)) throw new Error(`${label} is missing its dialect or objects list.`);
  for (const object of snapshot.objects) {
    if (!object.kind || !object.schema || !object.name) throw new Error(`${label} contains an object without kind, schema, or name.`);
  }
  return snapshot as Snapshot;
}

function classify(change: 'added' | 'removed' | 'modified', object: SchemaObject, before?: SchemaObject): ReviewChange {
  const label = objectLabel(object);
  if (change === 'removed' && ['table', 'view', 'column'].includes(object.kind)) {
    return { risk: 'high', object: label, title: `Removed ${object.kind}`, explanation: `Applications or stored queries may still depend on this ${object.kind}.` };
  }
  if (change === 'modified' && object.kind === 'column') {
    const becameRequired = before?.details?.nullable === true && object.details?.nullable === false;
    const typeChanged = before?.details?.data_type !== object.details?.data_type;
    if (becameRequired || typeChanged) return { risk: 'high', object: label, title: 'Changed column contract', explanation: 'The type or nullability became stricter; existing data or writes may no longer fit.' };
  }
  if (change === 'added' && object.kind === 'view') {
    return { risk: 'medium', object: label, title: 'Added ORM-invisible view', explanation: 'Many ORMs do not represent database view relationships completely.' };
  }
  if (change === 'modified') {
    return { risk: 'medium', object: label, title: `Modified ${object.kind}`, explanation: 'Its catalog definition differs from the expected migration state.' };
  }
  if (change === 'removed') {
    return { risk: 'medium', object: label, title: `Removed ${object.kind}`, explanation: 'Query behavior or integrity guarantees may have changed.' };
  }
  return { risk: 'low', object: label, title: `Added ${object.kind}`, explanation: 'This additive object should still be matched to an owning migration.' };
}

function compareSnapshots(before: Snapshot, after: Snapshot): ReviewChange[] {
  if (before.dialect !== after.dialect) throw new Error(`These snapshots use different dialects (${before.dialect} and ${after.dialect}).`);
  const oldObjects = new Map(before.objects.map((object) => [objectKey(object), object]));
  const newObjects = new Map(after.objects.map((object) => [objectKey(object), object]));
  const changes: ReviewChange[] = [];
  oldObjects.forEach((oldObject, key) => {
    const newObject = newObjects.get(key);
    if (!newObject) changes.push(classify('removed', oldObject));
    else if (JSON.stringify(oldObject.details ?? {}) !== JSON.stringify(newObject.details ?? {})) changes.push(classify('modified', newObject, oldObject));
  });
  newObjects.forEach((newObject, key) => {
    if (!oldObjects.has(key)) changes.push(classify('added', newObject));
  });
  const order = { high: 0, medium: 1, low: 2 };
  return changes.sort((a, b) => order[a.risk] - order[b.risk] || a.object.localeCompare(b.object));
}

function showError(message: string): void {
  if (!errorState || !emptyState || !resultState) return;
  errorState.textContent = message;
  errorState.hidden = false;
  emptyState.hidden = true;
  resultState.hidden = true;
}

function renderReview(changes: ReviewChange[]): void {
  if (!errorState || !emptyState || !resultState || !summary || !counts || !changeList) return;
  errorState.hidden = true;
  emptyState.hidden = true;
  resultState.hidden = false;
  changeList.replaceChildren();
  const riskCounts = { high: 0, medium: 0, low: 0 };
  changes.forEach((change) => {
    riskCounts[change.risk] += 1;
    const item = document.createElement('li');
    const badge = document.createElement('span');
    badge.className = `risk-tab ${change.risk}`;
    badge.textContent = `${change.risk} risk`;
    const title = document.createElement('h3');
    title.textContent = `${change.title} · ${change.object}`;
    const explanation = document.createElement('p');
    explanation.textContent = change.explanation;
    item.append(badge, title, explanation);
    changeList.append(item);
  });
  if (changes.length === 0) {
    summary.textContent = 'No drift detected';
    counts.textContent = 'The catalog layers match.';
  } else {
    summary.textContent = `${changes.length} difference${changes.length === 1 ? '' : 's'} found`;
    counts.textContent = `${riskCounts.high} high · ${riskCounts.medium} medium · ${riskCounts.low} low`;
  }
}

function resetDemo(showSampleReview = isDemoRoute): void {
  if (beforeInput) beforeInput.value = JSON.stringify(expected, null, 2);
  if (afterInput) afterInput.value = JSON.stringify(observed, null, 2);
  if (emptyState) emptyState.hidden = showSampleReview;
  if (errorState) errorState.hidden = true;
  if (resultState) resultState.hidden = true;
  if (showSampleReview) renderReview(compareSnapshots(expected, observed));
}

document.querySelector('#compare-button')?.addEventListener('click', () => {
  try {
    renderReview(compareSnapshots(parseSnapshot(beforeInput?.value ?? '', 'Expected snapshot'), parseSnapshot(afterInput?.value ?? '', 'Observed snapshot')));
  } catch (error) {
    showError(error instanceof Error ? error.message : 'The comparison could not be completed.');
  }
});
document.querySelectorAll<HTMLElement>('[data-reset-demo], #reset-demo').forEach((button) => {
  button.addEventListener('click', () => resetDemo());
});
resetDemo();

document.querySelectorAll<HTMLButtonElement>('[data-copy]').forEach((button) => {
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(button.dataset.copy ?? '');
      button.textContent = 'Copied';
      window.setTimeout(() => { button.textContent = 'Copy'; }, 1600);
    } catch {
      button.textContent = 'Select command';
    }
  });
});

type Verdict = { valid: boolean; reason: string; checkedAt: number };
const licenseStatus = document.querySelector<HTMLElement>('#license-status');

function readVerdict(): Verdict | null {
  try {
    return JSON.parse(localStorage.getItem(VERDICT_KEY) ?? 'null') as Verdict | null;
  } catch { return null; }
}

function showLicense(valid: boolean, message: string): void {
  document.documentElement.dataset.pro = valid ? 'unlocked' : 'locked';
  if (licenseStatus) licenseStatus.textContent = message;
}

async function verifyLicense(token: string): Promise<void> {
  if (!navigator.onLine) {
    const cached = readVerdict();
    showLicense(cached?.valid === true, cached?.valid ? 'Pro unlocked from the last verified license. Verification will resume online.' : 'You are offline. Reconnect to verify this license once.');
    return;
  }
  showLicense(false, 'Checking this license…');
  try {
    const response = await fetch(`${API_BASE}/api/v1/products/${PRODUCT}/verify?license=${encodeURIComponent(token)}`, { headers: { accept: 'application/json' } });
    if (!response.ok) throw new Error('The license service did not respond normally.');
    const result = await response.json() as { valid: boolean; reason: string };
    const verdict = { valid: result.valid, reason: result.reason, checkedAt: Date.now() };
    localStorage.setItem(VERDICT_KEY, JSON.stringify(verdict));
    if (result.valid) showLicense(true, 'Pro is unlocked on this device.');
    else showLicense(false, `License no longer active (${result.reason}). Checkout is closed; restore a different license below.`);
  } catch {
    const cached = readVerdict();
    showLicense(cached?.valid === true, cached?.valid ? 'Could not refresh the license; using the last valid verdict.' : 'Could not reach license verification. Your free tools remain available.');
  }
}

function startLicense(): void {
  const query = new URLSearchParams(location.search);
  const returnedLicense = query.get('license');
  if (returnedLicense) {
    localStorage.setItem(LICENSE_KEY, returnedLicense);
    query.delete('license');
    const cleanUrl = `${location.pathname}${query.size ? `?${query}` : ''}${location.hash}`;
    history.replaceState({}, '', cleanUrl);
  }
  const token = returnedLicense ?? localStorage.getItem(LICENSE_KEY);
  if (!token) return;
  const verdict = readVerdict();
  if (verdict?.valid) showLicense(true, 'Pro unlocked from your verified license.');
  else if (verdict) showLicense(false, `License no longer active (${verdict.reason}). Checkout is closed; restore a different license below.`);
  if (returnedLicense || !verdict || Date.now() - verdict.checkedAt >= DAY_MS) void verifyLicense(token);
}

document.querySelector<HTMLFormElement>('#license-form')?.addEventListener('submit', (event) => {
  event.preventDefault();
  const input = document.querySelector<HTMLInputElement>('#license-token');
  const token = input?.value.trim() ?? '';
  if (!token) return;
  localStorage.setItem(LICENSE_KEY, token);
  if (input) input.value = '';
  void verifyLicense(token);
});
if (!isDemoRoute) startLicense();

document.querySelectorAll<HTMLElement>('[data-build-id]').forEach((element) => {
  element.textContent = __SDS_BUILD_ID__;
});

const offlineStrip = document.querySelector<HTMLElement>('.offline-strip');
function updateNetworkState(): void { if (offlineStrip) offlineStrip.hidden = navigator.onLine; }
window.addEventListener('online', updateNetworkState);
window.addEventListener('offline', updateNetworkState);
updateNetworkState();

if ('serviceWorker' in navigator && location.protocol !== 'file:') {
  window.addEventListener('load', () => { void navigator.serviceWorker.register('/sw.js'); });
}
