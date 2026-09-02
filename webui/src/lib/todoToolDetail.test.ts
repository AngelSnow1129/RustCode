import test from 'node:test';
import assert from 'node:assert/strict';
import { commitTodoCall, todoToolDetail } from './todoToolDetail.ts';

test('todowrite update includes the task title from the successful plan', () => {
  const titles = new Map<number, string>();
  commitTodoCall(
    'todowrite',
    '{"todos":[{"content":"inspect oauth","status":"pending"},{"content":"write report","status":"in_progress"}]}',
    titles,
  );

  assert.equal(
    todoToolDetail('todowrite', '{"action":"update","id":2,"status":"completed"}', titles),
    '#2 write report → completed',
  );
});

test('failed calls can remain staged without changing task titles', () => {
  const titles = new Map<number, string>([[1, 'existing task']]);
  assert.equal(
    todoToolDetail('todowrite', '{"action":"update","id":1,"status":"completed"}', titles),
    '#1 existing task → completed',
  );
  assert.deepEqual([...titles], [[1, 'existing task']]);
});

test('todowrite full plans keep the compact task-count label', () => {
  // node tests have no localStorage, so translate() resolves to the zh default.
  assert.equal(
    todoToolDetail('todowrite', '{"todos":[{"content":"one","status":"pending"}]}', new Map()),
    '1 个任务',
  );
  assert.equal(
    todoToolDetail('todowrite', '{"todos":[{"content":"a"},{"content":"b"},{"content":"c"}]}', new Map()),
    '3 个任务',
  );
  assert.equal(
    todoToolDetail('todowrite', '{"action":"list"}', new Map()),
    '列出全部',
  );
});
