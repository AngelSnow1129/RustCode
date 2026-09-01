import type { AuthStatusResponse } from '../daemon/types';

export type AuthDisplayState = 'signed_in' | 'expired' | 'signed_out';

export function classifyAuthDisplayState(
  auth: Pick<AuthStatusResponse, 'logged_in' | 'expired'> | undefined,
): AuthDisplayState {
  if (auth?.expired) return 'expired';
  return auth?.logged_in ? 'signed_in' : 'signed_out';
}

/**
 * Whether the running daemon ships a managed sign-in service (GET
 * /auth/status `managed_available`). Open builds return false and older
 * daemons omit the field: fail closed, so every managed entry point stays
 * hidden until a daemon explicitly advertises the capability.
 */
export function managedLoginAvailable(
  auth: Pick<AuthStatusResponse, 'managed_available'> | undefined,
): boolean {
  return auth?.managed_available === true;
}
