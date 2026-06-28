import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Text,
  View,
  TextInput,
  Button,
  FlatList,
  TouchableOpacity,
  Alert,
  RefreshControl,
  ActivityIndicator,
  Modal,
  Pressable,
  StatusBar,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { API_URL } from './src/config';
import { FlatTask, Task, flattenTasks } from './src/tasks';
import { ThemeMode, createStyles, themes } from './src/theme';

const App = () => {
  const [themeMode, setThemeMode] = useState<ThemeMode>('light');
  const theme = themes[themeMode];
  const styles = useMemo(() => createStyles(theme), [theme]);

  const [tasks, setTasks] = useState<Task[]>([]);
  const [newTitle, setNewTitle] = useState('');
  const [refreshing, setRefreshing] = useState(false);
  const [subtaskParent, setSubtaskParent] = useState<Task | null>(null);
  const [subtaskTitle, setSubtaskTitle] = useState('');

  const flatTasks = useMemo(() => flattenTasks(tasks), [tasks]);

  const toggleTheme = () => {
    setThemeMode(current => (current === 'light' ? 'dark' : 'light'));
  };

  const fetchTasks = useCallback(async () => {
    try {
      const res = await fetch(API_URL);
      if (!res.ok) {
        throw new Error(`HTTP error! status: ${res.status}`);
      }
      const data: Task[] = await res.json();
      setTasks(data);
    } catch (error) {
      console.error('Error fetching tasks:', error);
      Alert.alert(
        'Error',
        'No se pudieron cargar las tareas. Asegúrate de que el backend esté corriendo y la URL sea correcta.',
      );
    }
  }, []);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await fetchTasks();
    setRefreshing(false);
  }, [fetchTasks]);

  const createTask = async (title: string, parentId?: number | null) => {
    const trimmed = title.trim();
    if (!trimmed) {
      Alert.alert('Advertencia', 'El título de la tarea no puede estar vacío.');
      return false;
    }

    try {
      const res = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          title: trimmed,
          parent_id: parentId ?? null,
        }),
      });
      if (!res.ok) {
        throw new Error(`HTTP error! status: ${res.status}`);
      }
      await fetchTasks();
      return true;
    } catch (error) {
      console.error('Error creating task:', error);
      Alert.alert('Error', 'No se pudo crear la tarea.');
      return false;
    }
  };

  const handleCreateRootTask = async () => {
    if (await createTask(newTitle)) {
      setNewTitle('');
    }
  };

  const openSubtaskModal = (parent: Task) => {
    setSubtaskParent(parent);
    setSubtaskTitle('');
  };

  const closeSubtaskModal = () => {
    setSubtaskParent(null);
    setSubtaskTitle('');
  };

  const handleCreateSubtask = async () => {
    if (!subtaskParent) {
      return;
    }
    if (await createTask(subtaskTitle, subtaskParent.id)) {
      closeSubtaskModal();
    }
  };

  const toggleTask = async (id: number) => {
    try {
      const res = await fetch(`${API_URL}/${id}/toggle`, { method: 'POST' });
      if (!res.ok) {
        throw new Error(`HTTP error! status: ${res.status}`);
      }
      fetchTasks();
    } catch (error) {
      console.error('Error toggling task:', error);
      Alert.alert('Error', 'No se pudo actualizar el estado de la tarea.');
    }
  };

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  const renderTask = ({ item }: { item: FlatTask }) => (
    <View style={[styles.taskItem, { marginLeft: item.depth * 20 }]}>
      <TouchableOpacity
        onPress={() => toggleTask(item.id)}
        style={styles.taskMain}
        accessibilityRole="button"
        accessibilityLabel={`Alternar tarea ${item.title}`}
      >
        <Text
          style={[
            styles.taskText,
            item.completed && styles.completedTaskText,
          ]}
        >
          {item.title}
        </Text>
        {item.completed && <Text style={styles.checkmark}>✓</Text>}
      </TouchableOpacity>
      <TouchableOpacity
        onPress={() => openSubtaskModal(item)}
        style={styles.addSubtaskButton}
        accessibilityLabel={`Agregar subtarea a ${item.title}`}
      >
        <Text style={styles.addSubtaskIcon}>+</Text>
      </TouchableOpacity>
    </View>
  );

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar
        barStyle={themeMode === 'light' ? 'dark-content' : 'light-content'}
        backgroundColor={theme.background}
      />
      <View style={styles.header}>
        <Text style={styles.title}>Gestor del Día a Día</Text>
        <View style={styles.headerActions}>
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
      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          value={newTitle}
          onChangeText={setNewTitle}
          placeholder="Nueva tarea..."
          placeholderTextColor={theme.placeholder}
        />
        <Button title="Agregar" onPress={handleCreateRootTask} color={theme.primary} />
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

      <Modal
        visible={subtaskParent !== null}
        transparent
        animationType="fade"
        onRequestClose={closeSubtaskModal}
      >
        <Pressable style={styles.modalBackdrop} onPress={closeSubtaskModal}>
          <Pressable style={styles.modalCard} onPress={() => {}}>
            <Text style={styles.modalTitle}>Nueva subtarea</Text>
            {subtaskParent && (
              <Text style={styles.modalSubtitle}>
                De: {subtaskParent.title}
              </Text>
            )}
            <TextInput
              style={styles.modalInput}
              value={subtaskTitle}
              onChangeText={setSubtaskTitle}
              placeholder="Título de la subtarea..."
              placeholderTextColor={theme.placeholder}
              autoFocus
            />
            <View style={styles.modalActions}>
              <Button title="Cancelar" onPress={closeSubtaskModal} color="#6c757d" />
              <Button title="Agregar" onPress={handleCreateSubtask} color={theme.primary} />
            </View>
          </Pressable>
        </Pressable>
      </Modal>
    </SafeAreaView>
  );
};

export default App;
