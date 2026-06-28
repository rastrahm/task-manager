import { Platform } from 'react-native';

const DEV_HOST =
  Platform.OS === 'android' ? '10.0.2.2' : 'localhost';

export const API_URL = `http://${DEV_HOST}:5040/tasks`;
