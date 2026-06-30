import { StyleSheet } from 'react-native';

export type ThemeMode = 'light' | 'dark';

export interface AppTheme {
  mode: ThemeMode;
  background: string;
  surface: string;
  text: string;
  textSecondary: string;
  textMuted: string;
  border: string;
  primary: string;
  primarySoft: string;
  success: string;
  modalBackdrop: string;
  placeholder: string;
  shadow: string;
  iconButtonBg: string;
}

export const themes: Record<ThemeMode, AppTheme> = {
  light: {
    mode: 'light',
    background: '#f8f8f8',
    surface: '#ffffff',
    text: '#333333',
    textSecondary: '#666666',
    textMuted: '#888888',
    border: '#dddddd',
    primary: '#007bff',
    primarySoft: '#e7f1ff',
    success: '#28a745',
    modalBackdrop: 'rgba(0, 0, 0, 0.45)',
    placeholder: '#666666',
    shadow: '#000000',
    iconButtonBg: '#ffffff',
  },
  dark: {
    mode: 'dark',
    background: '#121212',
    surface: '#1e1e1e',
    text: '#f0f0f0',
    textSecondary: '#b0b0b0',
    textMuted: '#888888',
    border: '#333333',
    primary: '#4dabf7',
    primarySoft: '#1a2a3a',
    success: '#51cf66',
    modalBackdrop: 'rgba(0, 0, 0, 0.7)',
    placeholder: '#999999',
    shadow: '#000000',
    iconButtonBg: '#2a2a2a',
  },
};

export function createStyles(theme: AppTheme) {
  return StyleSheet.create({
    container: {
      flex: 1,
      backgroundColor: theme.background,
      alignItems: 'center',
      paddingTop: 20,
    },
    header: {
      flexDirection: 'row',
      alignItems: 'center',
      justifyContent: 'space-between',
      width: '90%',
      marginBottom: 20,
    },
    title: {
      fontSize: 28,
      fontWeight: 'bold',
      color: theme.text,
      flex: 1,
    },
    headerActions: {
      flexDirection: 'row',
      alignItems: 'center',
      gap: 8,
    },
    iconButton: {
      width: 44,
      height: 44,
      borderRadius: 22,
      backgroundColor: theme.iconButtonBg,
      alignItems: 'center',
      justifyContent: 'center',
      shadowColor: theme.shadow,
      shadowOffset: { width: 0, height: 1 },
      shadowOpacity: theme.mode === 'light' ? 0.1 : 0.3,
      shadowRadius: 2,
      elevation: 2,
    },
    refreshIcon: {
      fontSize: 26,
      color: theme.primary,
      fontWeight: 'bold',
    },
    themeIcon: {
      fontSize: 22,
    },
    newTaskRow: {
      width: '90%',
      marginBottom: 16,
    },
    taskList: {
      width: '90%',
    },
    taskListContent: {
      paddingBottom: 20,
    },
    taskItem: {
      flexDirection: 'row',
      alignItems: 'center',
      padding: 12,
      borderBottomWidth: 1,
      borderBottomColor: theme.border,
      backgroundColor: theme.surface,
      borderRadius: 10,
      marginBottom: 10,
      shadowColor: theme.shadow,
      shadowOffset: { width: 0, height: 1 },
      shadowOpacity: theme.mode === 'light' ? 0.08 : 0.2,
      shadowRadius: 1,
      elevation: 1,
    },
    taskMain: {
      flex: 1,
      flexDirection: 'row',
      alignItems: 'center',
    },
    taskContent: {
      flex: 1,
    },
    taskText: {
      fontSize: 18,
      color: theme.text,
    },
    taskDescription: {
      fontSize: 14,
      color: theme.textSecondary,
      marginTop: 4,
    },
    taskMeta: {
      fontSize: 12,
      color: theme.textMuted,
      marginTop: 4,
    },
    completedTaskText: {
      textDecorationLine: 'line-through',
      color: theme.textMuted,
      fontStyle: 'italic',
    },
    checkmark: {
      fontSize: 22,
      color: theme.success,
      marginLeft: 15,
    },
    editButton: {
      width: 36,
      height: 36,
      borderRadius: 18,
      backgroundColor: theme.iconButtonBg,
      alignItems: 'center',
      justifyContent: 'center',
      marginLeft: 8,
      borderWidth: 1,
      borderColor: theme.border,
    },
    editIcon: {
      fontSize: 16,
      color: theme.textSecondary,
    },
    addSubtaskButton: {
      width: 36,
      height: 36,
      borderRadius: 18,
      backgroundColor: theme.primarySoft,
      alignItems: 'center',
      justifyContent: 'center',
      marginLeft: 8,
    },
    addSubtaskIcon: {
      fontSize: 22,
      color: theme.primary,
      fontWeight: 'bold',
      lineHeight: 24,
    },
    emptyText: {
      textAlign: 'center',
      color: theme.textMuted,
      marginTop: 24,
      fontSize: 16,
    },
    modalBackdrop: {
      flex: 1,
      backgroundColor: theme.modalBackdrop,
      justifyContent: 'center',
      padding: 24,
    },
    modalCard: {
      backgroundColor: theme.surface,
      borderRadius: 12,
      padding: 20,
    },
    formModalCard: {
      backgroundColor: theme.surface,
      borderRadius: 12,
      padding: 20,
      maxHeight: '90%',
    },
    formLabel: {
      fontSize: 14,
      fontWeight: '600',
      color: theme.textSecondary,
      marginBottom: 6,
    },
    formTextArea: {
      minHeight: 88,
      marginBottom: 16,
    },
    priorityRow: {
      flexDirection: 'row',
      flexWrap: 'wrap',
      gap: 8,
      marginBottom: 16,
    },
    priorityChip: {
      paddingHorizontal: 12,
      paddingVertical: 8,
      borderRadius: 16,
      borderWidth: 1,
      borderColor: theme.border,
      backgroundColor: theme.background,
    },
    priorityChipActive: {
      borderColor: theme.primary,
      backgroundColor: theme.primarySoft,
    },
    priorityChipText: {
      fontSize: 13,
      color: theme.textSecondary,
      textTransform: 'capitalize',
    },
    modalTitle: {
      fontSize: 20,
      fontWeight: 'bold',
      color: theme.text,
      marginBottom: 8,
    },
    modalSubtitle: {
      fontSize: 14,
      color: theme.textSecondary,
      marginBottom: 16,
    },
    modalInput: {
      borderWidth: 1,
      borderColor: theme.border,
      borderRadius: 8,
      padding: 12,
      fontSize: 16,
      color: theme.text,
      backgroundColor: theme.background,
      marginBottom: 16,
    },
    modalActions: {
      flexDirection: 'row',
      justifyContent: 'space-between',
      gap: 12,
    },
  });
}
