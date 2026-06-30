import React, { useMemo, useState } from 'react';
import {
  ActivityIndicator,
  Alert,
  Button,
  Text,
  TextInput,
  View,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { apiClient } from './apiClient';
import { AppTheme, createStyles } from './theme';

interface LoginScreenProps {
  theme: AppTheme;
  onSuccess: () => void;
}

export function LoginScreen({ theme, onSuccess }: LoginScreenProps) {
  const styles = useMemo(() => createStyles(theme), [theme]);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState('');

  const handleLogin = async () => {
    if (!username.trim() || !password) {
      Alert.alert('Advertencia', 'Usuario y contraseña son obligatorios.');
      return;
    }

    setLoading(true);
    setStatus('Conectando con el servidor…');
    try {
      await apiClient.login(username, password);
      onSuccess();
    } catch (error) {
      const message =
        error instanceof Error
          ? error.message
          : 'No se pudo iniciar sesión.';
      setStatus('');
      Alert.alert('Error de autenticación', message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <SafeAreaView style={styles.loginContainer}>
      <View style={styles.loginCard}>
        <Text style={styles.loginTitle}>Gestor del Día a Día</Text>
        <Text style={styles.formLabel}>Usuario</Text>
        <TextInput
          style={styles.modalInput}
          placeholder="Nombre de usuario"
          placeholderTextColor={theme.placeholder}
          autoCapitalize="none"
          autoCorrect={false}
          value={username}
          onChangeText={setUsername}
          editable={!loading}
        />
        <Text style={styles.formLabel}>Contraseña</Text>
        <TextInput
          style={styles.modalInput}
          placeholder="Contraseña"
          placeholderTextColor={theme.placeholder}
          secureTextEntry
          value={password}
          onChangeText={setPassword}
          onSubmitEditing={handleLogin}
          editable={!loading}
        />
        {status ? <Text style={styles.loginStatus}>{status}</Text> : null}
        <View style={styles.loginActions}>
          {loading ? (
            <ActivityIndicator color={theme.primary} />
          ) : (
            <Button title="Entrar" onPress={handleLogin} color={theme.primary} />
          )}
        </View>
      </View>
    </SafeAreaView>
  );
}
