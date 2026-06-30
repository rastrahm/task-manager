/**
 * Componente raíz: arranque, sesión y enrutado entre login y tareas.
 * @module App
 */

import { useEffect, useState } from 'react';
import { apiClient } from './apiClient';
import LoginScreen from './LoginScreen';
import TaskApp from './TaskApp';
import './App.css';

/**
 * Fases del flujo de arranque y autenticación.
 * @typedef {'booting' | 'login' | 'main'} AppPhase
 */

/**
 * Orquesta la restauración de sesión y muestra login o la pantalla principal.
 * @returns {JSX.Element}
 */
export default function App() {
  const [phase, setPhase] = useState('booting');

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

  if (phase === 'booting') {
    return (
      <div className="app-boot">
        <div className="app-boot-spinner" aria-label="Cargando" />
      </div>
    );
  }

  if (phase === 'login') {
    return <LoginScreen onSuccess={() => setPhase('main')} />;
  }

  return (
    <TaskApp
      onLogout={async () => {
        await apiClient.logout();
        setPhase('login');
      }}
      onSessionExpired={() => {
        apiClient.clearLocalSession();
        setPhase('login');
        alert('Sesión expirada. Vuelve a iniciar sesión.');
      }}
    />
  );
}
