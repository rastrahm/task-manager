import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Text,
  View,
  Button,
  FlatList,
  TouchableOpacity,
  Alert,
  RefreshControl,
  ActivityIndicator,
  StatusBar,
  Modal,
  Pressable,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { createTask, fetchTasks as loadTasks, toggleTask, updateTask } from './src/api';
import { apiClient, SessionExpiredError } from './src/apiClient';
import { LoginScreen } from './src/LoginScreen';
import { metadataToJson, metadataSummary, parseMetadata } from './src/metadata';
import { TaskFormModal, TaskFormMode } from './src/TaskFormModal';
import { FlatTask, Task, flattenTasks } from './src/tasks';
import { ThemeMode, createStyles, themes } from './src/theme';
import { UserAdminModal } from './src/UserAdminModal';

type AppPhase = 'booting' | 'login' | 'main';

const App = () => {
  const [themeMode, setThemeMode] = useState<ThemeMode>('light');
  const theme = themes[themeMode];
  const styles = useMemo(() => createStyles(theme), [theme]);

  const [phase, setPhase] = useState<AppPhase>('booting');
  const [tasks, setTasks] = useState<Task[]>([]);
  const [refreshing, setRefreshing] = useState(false);
  const [formMode, setFormMode] = useState<TaskFormMode | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const [userAdminOpen, setUserAdminOpen] = useState(false);

  const flatTasks = useMemo(() => flattenTasks(tasks), [tasks]);
  const username = apiClient.username ?? 'usuario';
  const isAdmin = apiClient.isAdmin;

  const handleSessionExpired = useCallback(() => {
    apiClient.clearLocalSession();
    setPhase('login');
    Alert.alert('Sesión expirada', 'Vuelve a iniciar sesión.');
  }, []);

  const handleApiError = useCallback(
    (error: unknown, fallback: string) => {
      if (error instanceof SessionExpiredError) {
        handleSessionExpired();
        return;
      }
      Alert.alert('Error', error instanceof Error ? error.message : fallback);
    },
    [handleSessionExpired],
  );

  const toggleTheme = () => {
    setThemeMode(current => (current === 'light' ? 'dark' : 'light'));
  };

  const fetchTasks = useCallback(async () => {
    try {
      const data = await loadTasks();
      setTasks(data);
    } catch (error) {
      console.error('Error fetching tasks:', error);
      handleApiError(
        error,
        'No se pudieron cargar las tareas. Asegúrate de que el backend esté corriendo.',
      );
    }
  }, [handleApiError]);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await fetchTasks();
    setRefreshing(false);
  }, [fetchTasks]);

  const handleFormSubmit = async (values: {
    title: string;
    description?: string | null;
    metadata: ReturnType<typeof parseMetadata>;
    parentId?: number | null;
    task?: Task;
  }) => {
    try {
      if (values.task) {
        await updateTask(values.task.id, {
          title: values.title,
          description: values.description ?? null,
          completed: values.task.completed,
          metadata: metadataToJson(values.metadata),
          parent_id: values.parentId ?? null,
        });
      } else {
        await createTask({
          title: values.title,
          description: values.description ?? null,
          metadata: metadataToJson(values.metadata),
          parent_id: values.parentId ?? null,
        });
      }
      await fetchTasks();
    } catch (error) {
      console.error('Error saving task:', error);
      handleApiError(error, 'No se pudo guardar la tarea.');
      throw error;
    }
  };

  const handleToggleTask = async (id: number) => {
    try {
      await toggleTask(id);
      fetchTasks();
    } catch (error) {
      console.error('Error toggling task:', error);
      handleApiError(error, 'No se pudo actualizar el estado de la tarea.');
    }
  };

  const handleLogout = async () => {
    setMenuOpen(false);
    await apiClient.logout();
    setTasks([]);
    setPhase('login');
  };

  useEffect(() => {
    (async () => {
      const session = await apiClient.restoreSession();
      if (!session) {
        setPhase('login');
        return;
      }
      try {
        await apiClient.refreshSessionIfNeeded();
        setPhase('main');
      } catch {
        apiClient.clearLocalSession();
        setPhase('login');
      }
    })();
  }, []);

  useEffect(() => {
    if (phase === 'main') {
      void fetchTasks();
    }
  }, [phase, fetchTasks]);

  const renderTask = ({ item }: { item: FlatTask }) => {
    const meta = parseMetadata(item.metadata);
    const metaLabel = metadataSummary(meta);

    return (
      <View style={[styles.taskItem, { marginLeft: item.depth * 20 }]}>
        <TouchableOpacity
          onPress={() => handleToggleTask(item.id)}
          style={styles.taskMain}
          accessibilityRole="button"
          accessibilityLabel={`Alternar tarea ${item.title}`}
        >
          <View style={styles.taskContent}>
            <Text
              style={[
                styles.taskText,
                item.completed && styles.completedTaskText,
              ]}
            >
              {item.title}
            </Text>
            {item.description ? (
              <Text style={styles.taskDescription} numberOfLines={2}>
                {item.description}
              </Text>
            ) : null}
            {metaLabel ? (
              <Text style={styles.taskMeta} numberOfLines={1}>
                {metaLabel}
              </Text>
            ) : null}
          </View>
          {item.completed && <Text style={styles.checkmark}>✓</Text>}
        </TouchableOpacity>
        <TouchableOpacity
          onPress={() =>
            setFormMode({ kind: 'edit', task: item })
          }
          style={styles.editButton}
          accessibilityLabel={`Editar tarea ${item.title}`}
        >
          <Text style={styles.editIcon}>✎</Text>
        </TouchableOpacity>
        <TouchableOpacity
          onPress={() =>
            setFormMode({
              kind: 'create',
              parentId: item.id,
              parentTitle: item.title,
            })
          }
          style={styles.addSubtaskButton}
          accessibilityLabel={`Agregar subtarea a ${item.title}`}
        >
          <Text style={styles.addSubtaskIcon}>+</Text>
        </TouchableOpacity>
      </View>
    );
  };

  if (phase === 'booting') {
    return (
      <SafeAreaView style={styles.loginContainer}>
        <ActivityIndicator size="large" color={theme.primary} />
      </SafeAreaView>
    );
  }

  if (phase === 'login') {
    return (
      <LoginScreen
        theme={theme}
        onSuccess={() => setPhase('main')}
      />
    );
  }

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar
        barStyle={themeMode === 'light' ? 'dark-content' : 'light-content'}
        backgroundColor={theme.background}
      />
      <View style={styles.header}>
        <View style={styles.headerTitleBlock}>
          <Text style={styles.title}>Gestor del Día a Día</Text>
          <Text style={styles.sessionLabel}>Sesión: {username}</Text>
          {isAdmin ? (
            <Text style={styles.adminBadge}>Administrador</Text>
          ) : null}
        </View>
        <View style={styles.headerActions}>
          <TouchableOpacity
            onPress={() => setMenuOpen(true)}
            style={styles.iconButton}
            accessibilityLabel="Menú de usuario"
          >
            <Text style={styles.refreshIcon}>☰</Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={toggleTheme}
            style={styles.iconButton}
            accessibilityLabel={
              themeMode === 'light' ? 'Activar tema oscuro' : 'Activar tema claro'
            }
          >
            <Text style={styles.themeIcon}>{themeMode === 'light' ? '🌙' : '☀️'}</Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={onRefresh}
            disabled={refreshing}
            style={styles.iconButton}
            accessibilityLabel="Actualizar tareas"
          >
            {refreshing ? (
              <ActivityIndicator size="small" color={theme.primary} />
            ) : (
              <Text style={styles.refreshIcon}>↻</Text>
            )}
          </TouchableOpacity>
        </View>
      </View>
      <View style={styles.newTaskRow}>
        <Button
          title="Nueva tarea"
          onPress={() => setFormMode({ kind: 'create' })}
          color={theme.primary}
        />
      </View>
      <FlatList
        data={flatTasks}
        keyExtractor={item => item.id.toString()}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={onRefresh}
            colors={[theme.primary]}
            tintColor={theme.primary}
          />
        }
        renderItem={renderTask}
        style={styles.taskList}
        contentContainerStyle={styles.taskListContent}
        ListEmptyComponent={
          <Text style={styles.emptyText}>No hay tareas todavía.</Text>
        }
      />

      <TaskFormModal
        visible={formMode !== null}
        mode={formMode}
        theme={theme}
        onClose={() => setFormMode(null)}
        onSubmit={handleFormSubmit}
      />

      <Modal
        visible={menuOpen}
        transparent
        animationType="fade"
        onRequestClose={() => setMenuOpen(false)}
      >
        <Pressable style={styles.menuBackdrop} onPress={() => setMenuOpen(false)}>
          <Pressable style={styles.menuCard} onPress={() => undefined}>
            <Text style={styles.menuTitle}>{username}</Text>
            {isAdmin ? (
              <TouchableOpacity
                style={styles.menuItem}
                onPress={() => {
                  setMenuOpen(false);
                  setUserAdminOpen(true);
                }}
              >
                <Text style={styles.menuItemText}>Administrar usuarios</Text>
              </TouchableOpacity>
            ) : null}
            <TouchableOpacity style={styles.menuItem} onPress={() => void handleLogout()}>
              <Text style={[styles.menuItemText, styles.menuItemDanger]}>
                Cerrar sesión
              </Text>
            </TouchableOpacity>
          </Pressable>
        </Pressable>
      </Modal>

      <UserAdminModal
        visible={userAdminOpen}
        theme={theme}
        onClose={() => setUserAdminOpen(false)}
      />
    </SafeAreaView>
  );
};

export default App;
