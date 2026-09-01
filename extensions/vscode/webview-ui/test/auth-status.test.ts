import assert from 'node:assert/strict';
import { classifyAuthDisplayState, managedLoginAvailable } from '../../src/auth/status';

assert.equal(
  classifyAuthDisplayState({ logged_in: true, expired: true }),
  'expired',
);
assert.equal(
  classifyAuthDisplayState({ logged_in: true, expired: false }),
  'signed_in',
);
assert.equal(
  classifyAuthDisplayState({ logged_in: false, expired: false }),
  'signed_out',
);
assert.equal(classifyAuthDisplayState(undefined), 'signed_out');

// --- managedLoginAvailable (build-capability gate) --------------------

assert.equal(
  managedLoginAvailable({ managed_available: true }),
  true,
  'daemon advertising the capability -> managed UI enabled',
);
assert.equal(
  managedLoginAvailable({ managed_available: false }),
  false,
  'open build -> managed UI hidden',
);
assert.equal(
  managedLoginAvailable(undefined),
  false,
  'missing status -> fail closed',
);
assert.equal(
  // Older daemons omit the field entirely.
  managedLoginAvailable({ managed_available: undefined }),
  false,
  'absent managed_available -> fail closed',
);
