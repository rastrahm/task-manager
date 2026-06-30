/**
 * @format
 */

import React from 'react';
import ReactTestRenderer from 'react-test-renderer';
import App from '../App';

jest.mock('@react-native-async-storage/async-storage', () =>
  require('@react-native-async-storage/async-storage/jest/async-storage-mock'),
);

jest.mock('../src/apiClient', () => ({
  SessionExpiredError: class SessionExpiredError extends Error {},
  apiClient: {
    username: null,
    isAdmin: false,
    restoreSession: jest.fn().mockResolvedValue(null),
    refreshSessionIfNeeded: jest.fn().mockResolvedValue(undefined),
    clearLocalSession: jest.fn(),
    logout: jest.fn().mockResolvedValue(undefined),
    login: jest.fn().mockResolvedValue(undefined),
  },
}));

jest.mock('../src/api', () => ({
  fetchTasks: jest.fn().mockResolvedValue([]),
  createTask: jest.fn(),
  updateTask: jest.fn(),
  toggleTask: jest.fn(),
}));

test('renders correctly', async () => {
  await ReactTestRenderer.act(async () => {
    ReactTestRenderer.create(<App />);
  });
});
