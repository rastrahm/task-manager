import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Alert,
  Button,
  FlatList,
  Modal,
  Pressable,
  Switch,
  Text,
  TextInput,
  TouchableOpacity,
  View,
} from 'react-native';
import { apiClient } from './apiClient';
import { User } from './models';
import { AppTheme, createStyles } from './theme';

interface UserAdminModalProps {
  visible: boolean;
  theme: AppTheme;
  onClose: () => void;
}

type UserFormMode =
  | { kind: 'create' }
  | { kind: 'edit'; user: User };

export function UserAdminModal({ visible, theme, onClose }: UserAdminModalProps) {
  const styles = useMemo(() => createStyles(theme), [theme]);
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(false);
  const [formMode, setFormMode] = useState<UserFormMode | null>(null);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [isAdmin, setIsAdmin] = useState(false);
  const [isActive, setIsActive] = useState(true);
  const [saving, setSaving] = useState(false);

  const reloadUsers = useCallback(async () => {
    setLoading(true);
    try {
      const fetched = await apiClient.listUsers();
      setUsers(fetched);
    } catch (error) {
      Alert.alert(
        'Error',
        error instanceof Error ? error.message : 'No se pudieron cargar los usuarios.',
      );
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (visible) {
      void reloadUsers();
      setFormMode(null);
    }
  }, [visible, reloadUsers]);

  useEffect(() => {
    if (!formMode) {
      return;
    }
    if (formMode.kind === 'edit') {
      setUsername(formMode.user.username);
      setPassword('');
      setIsAdmin(formMode.user.is_admin);
      setIsActive(formMode.user.is_active);
      return;
    }
    setUsername('');
    setPassword('');
    setIsAdmin(false);
    setIsActive(true);
  }, [formMode]);

  const handleDelete = (user: User) => {
    Alert.alert(
      'Eliminar usuario',
      `¿Eliminar al usuario «${user.username}»? Esta acción no se puede deshacer.`,
      [
        { text: 'Cancelar', style: 'cancel' },
        {
          text: 'Eliminar',
          style: 'destructive',
          onPress: async () => {
            try {
              await apiClient.deleteUser(user.id);
              await reloadUsers();
            } catch (error) {
              Alert.alert(
                'Error',
                error instanceof Error ? error.message : 'No se pudo eliminar el usuario.',
              );
            }
          },
        },
      ],
    );
  };

  const handleSaveUser = async () => {
    if (!username.trim()) {
      Alert.alert('Advertencia', 'El nombre de usuario es obligatorio.');
      return;
    }
    if (formMode?.kind === 'create' && !password) {
      Alert.alert('Advertencia', 'La contraseña es obligatoria.');
      return;
    }

    setSaving(true);
    try {
      if (formMode?.kind === 'edit') {
        await apiClient.updateUser(formMode.user.id, {
          username: username.trim(),
          password: password || undefined,
          is_admin: isAdmin,
          is_active: isActive,
        });
      } else {
        await apiClient.createUser({
          username: username.trim(),
          password,
          is_admin: isAdmin,
        });
      }
      setFormMode(null);
      await reloadUsers();
    } catch (error) {
      Alert.alert(
        'Error',
        error instanceof Error ? error.message : 'No se pudo guardar el usuario.',
      );
    } finally {
      setSaving(false);
    }
  };

  const renderUser = ({ item }: { item: User }) => (
    <View style={styles.userRow}>
      <View style={styles.userRowMain}>
        <Text style={styles.userRowTitle}>
          {item.username}
          {item.is_admin ? ' (administrador)' : ''}
        </Text>
        <Text style={styles.userRowMeta}>
          {item.is_active ? 'Activo' : 'Inactivo'}
        </Text>
      </View>
      <View style={styles.userRowActions}>
        <TouchableOpacity
          onPress={() => setFormMode({ kind: 'edit', user: item })}
          style={styles.userActionButton}
        >
          <Text style={styles.userActionText}>Editar</Text>
        </TouchableOpacity>
        <TouchableOpacity
          onPress={() => handleDelete(item)}
          style={styles.userActionButton}
        >
          <Text style={[styles.userActionText, styles.userActionDanger]}>
            Eliminar
          </Text>
        </TouchableOpacity>
      </View>
    </View>
  );

  return (
    <Modal visible={visible} animationType="slide" onRequestClose={onClose}>
      <View style={styles.adminContainer}>
        <View style={styles.adminHeader}>
          <Text style={styles.modalTitle}>Administración de usuarios</Text>
          <Button title="Cerrar" onPress={onClose} color={theme.primary} />
        </View>

        <View style={styles.adminToolbar}>
          <Button
            title="Actualizar"
            onPress={() => void reloadUsers()}
            color={theme.primary}
          />
          <Button
            title="Nuevo usuario"
            onPress={() => setFormMode({ kind: 'create' })}
            color={theme.primary}
          />
        </View>

        <FlatList
          data={users}
          keyExtractor={item => item.id.toString()}
          renderItem={renderUser}
          refreshing={loading}
          onRefresh={() => void reloadUsers()}
          ListEmptyComponent={
            <Text style={styles.emptyText}>
              {loading ? 'Cargando…' : 'No hay usuarios.'}
            </Text>
          }
          contentContainerStyle={styles.adminListContent}
        />

        <Modal
          visible={formMode !== null}
          transparent
          animationType="fade"
          onRequestClose={() => setFormMode(null)}
        >
          <Pressable
            style={styles.modalBackdrop}
            onPress={() => setFormMode(null)}
          >
            <Pressable style={styles.formModalCard} onPress={() => undefined}>
              <Text style={styles.modalTitle}>
                {formMode?.kind === 'edit' ? 'Editar usuario' : 'Nuevo usuario'}
              </Text>
              <Text style={styles.formLabel}>Usuario *</Text>
              <TextInput
                style={styles.modalInput}
                value={username}
                onChangeText={setUsername}
                autoCapitalize="none"
                editable={!saving}
              />
              <Text style={styles.formLabel}>
                {formMode?.kind === 'edit'
                  ? 'Nueva contraseña (vacío = no cambiar)'
                  : 'Contraseña *'}
              </Text>
              <TextInput
                style={styles.modalInput}
                value={password}
                onChangeText={setPassword}
                secureTextEntry
                editable={!saving}
              />
              <View style={styles.switchRow}>
                <Text style={styles.formLabel}>Administrador</Text>
                <Switch value={isAdmin} onValueChange={setIsAdmin} />
              </View>
              {formMode?.kind === 'edit' ? (
                <View style={styles.switchRow}>
                  <Text style={styles.formLabel}>Cuenta activa</Text>
                  <Switch value={isActive} onValueChange={setIsActive} />
                </View>
              ) : null}
              <View style={styles.modalActions}>
                <Button
                  title="Cancelar"
                  onPress={() => setFormMode(null)}
                  color={theme.textMuted}
                />
                <Button
                  title={saving ? 'Guardando…' : 'Guardar'}
                  onPress={() => void handleSaveUser()}
                  color={theme.primary}
                  disabled={saving}
                />
              </View>
            </Pressable>
          </Pressable>
        </Modal>
      </View>
    </Modal>
  );
}
