// Per-turn action targets for the assistant toolbar: "restore to before this
// turn" and "regenerate this turn". Both are built on the same `/undo N`
// primitive as the `/undo` slash command — the server truncates the session to
// everything BEFORE user prompt N (`undo_snapshot_to_prompt`), so regenerating
// is "undo this turn, then resend the same prompt".

import type { ImageData } from '../api.ts';

/** Minimal shape needed here — structurally satisfied by Chat's local
 *  `Message`, so the component does not have to export it. */
export interface TurnActionMessage {
  role: string;
  parts?: readonly { kind: string; text?: string }[];
  images?: ImageData[];
}

export interface TurnActionTarget {
  /** 1-based ordinal among real user prompts — exactly what `/undo N` takes. */
  promptN: number;
  promptText: string;
  promptImages: ImageData[];
  /** Newest turn in the conversation. Only it can be regenerated in place:
   *  regenerating an older turn would have to delete everything after it. */
  isLastTurn: boolean;
}

/**
 * Map every ASSISTANT TURN-END index to the user prompt that opened that turn.
 *
 * Keyed by the same indices as `assistantTurnEndFlags(roles)` so the caller can
 * render the actions on exactly the row that already owns the copy button.
 * System notices are transparent (they never open a turn) and internal/synthetic
 * prompts are already filtered out of the displayed list upstream.
 */
export function assistantTurnActions(
  messages: readonly TurnActionMessage[],
  ends: readonly boolean[],
): Map<number, TurnActionTarget> {
  const targets = new Map<number, TurnActionTarget>();
  let lastEnd = -1;
  for (let i = ends.length - 1; i >= 0; i -= 1) {
    if (ends[i]) {
      lastEnd = i;
      break;
    }
  }
  let promptN = 0;
  let current: TurnActionTarget | null = null;
  for (let i = 0; i < messages.length; i += 1) {
    const message = messages[i];
    if (message.role === 'user') {
      promptN += 1;
      current = {
        promptN,
        promptText: (message.parts ?? [])
          .filter((part) => part.kind === 'text')
          .map((part) => part.text ?? '')
          .join(''),
        promptImages: message.images ?? [],
        isLastTurn: false,
      };
      continue;
    }
    if (message.role === 'assistant' && ends[i] && current) {
      targets.set(i, { ...current, isLastTurn: i === lastEnd });
    }
  }
  return targets;
}
