/**
 * Pantalla de inicio de sesión contra `POST /auth/login`.
 * @module LoginScreen
 */

import React, { useState } from 'react';
import { apiClient } from './apiClient';
import './Auth.css';

/**
 * @param {object} props
 * @param {Function} props.onSuccess - Callback tras autenticación correcta.
 * @returns {JSX.Element}
 */
export default function LoginScreen({ onSuccess }) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState('');
  const [error, setError] = useState('');

  const handleLogin = async (event) => {
    event?.preventDefault();
    if (!username.trim() || !password) {
      setError('Usuario y contraseña son obligatorios.');
      return;
    }

    setLoading(true);
    setStatus('Conectando con el servidor…');
    setError('');
    try {
      await apiClient.login(username, password);
      onSuccess();
    } catch (err) {
      setStatus('');
      setError(err instanceof Error ? err.message : 'No se pudo iniciar sesión.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-page">
      <form className="login-card" onSubmit={handleLogin}>
        <h1 className="login-title">Gestor del Día a Día</h1>

        <label className="auth-label" htmlFor="login-username">
          Usuario
        </label>
        <input
          id="login-username"
          className="auth-input"
          type="text"
          placeholder="Nombre de usuario"
          autoComplete="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          disabled={loading}
        />

        <label className="auth-label" htmlFor="login-password">
          Contraseña
        </label>
        <input
          id="login-password"
          className="auth-input"
          type="password"
          placeholder="Contraseña"
          autoComplete="current-password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          disabled={loading}
        />

        {status ? <p className="login-status">{status}</p> : null}
        {error ? <p className="login-error" role="alert">{error}</p> : null}

        <div className="login-actions">
          <button type="submit" className="btn-primary" disabled={loading}>
            {loading ? 'Conectando…' : 'Entrar'}
          </button>
        </div>
      </form>
    </div>
  );
}
