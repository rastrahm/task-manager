import React, { useCallback, useEffect, useState } from 'react';
import { apiClient } from './apiClient';
import './Auth.css';

export default function UserAdminModal({ open, onClose }) {
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(false);
  const [formMode, setFormMode] = useState(null);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [isAdmin, setIsAdmin] = useState(false);
  const [isActive, setIsActive] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  const reloadUsers = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const fetched = await apiClient.listUsers();
      setUsers(fetched);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'No se pudieron cargar los usuarios.');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (open) {
      void reloadUsers();
      setFormMode(null);
    }
  }, [open, reloadUsers]);

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

  if (!open) {
    return null;
  }

  const handleDelete = (user) => {
    if (!window.confirm(`¿Eliminar al usuario «${user.username}»? Esta acción no se puede deshacer.`)) {
      return;
    }
    void (async () => {
      try {
        await apiClient.deleteUser(user.id);
        await reloadUsers();
      } catch (err) {
        alert(err instanceof Error ? err.message : 'No se pudo eliminar el usuario.');
      }
    })();
  };

  const handleSaveUser = async (event) => {
    event.preventDefault();
    if (!username.trim()) {
      alert('El nombre de usuario es obligatorio.');
      return;
    }
    if (formMode?.kind === 'create' && !password) {
      alert('La contraseña es obligatoria.');
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
    } catch (err) {
      alert(err instanceof Error ? err.message : 'No se pudo guardar el usuario.');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="task-modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="user-admin-title">
      <div className="user-admin-card">
        <div className="user-admin-header">
          <h3 id="user-admin-title">Administración de usuarios</h3>
          <button type="button" className="btn-secondary" onClick={onClose}>
            Cerrar
          </button>
        </div>

        <div className="user-admin-toolbar">
          <button type="button" className="btn-secondary" onClick={() => void reloadUsers()} disabled={loading}>
            Actualizar
          </button>
          <button type="button" className="btn-primary" onClick={() => setFormMode({ kind: 'create' })}>
            Nuevo usuario
          </button>
        </div>

        {error ? <p className="login-error" role="alert">{error}</p> : null}

        <ul className="user-admin-list">
          {users.length === 0 ? (
            <li className="user-admin-empty">{loading ? 'Cargando…' : 'No hay usuarios.'}</li>
          ) : (
            users.map((user) => (
              <li key={user.id} className="user-admin-row">
                <div>
                  <strong>
                    {user.username}
                    {user.is_admin ? ' (administrador)' : ''}
                  </strong>
                  <p className="user-admin-meta">{user.is_active ? 'Activo' : 'Inactivo'}</p>
                </div>
                <div className="user-admin-row-actions">
                  <button type="button" className="btn-action btn-edit" onClick={() => setFormMode({ kind: 'edit', user })}>
                    Editar
                  </button>
                  <button type="button" className="btn-action btn-danger" onClick={() => handleDelete(user)}>
                    Eliminar
                  </button>
                </div>
              </li>
            ))
          )}
        </ul>

        {formMode ? (
          <div className="user-admin-form-backdrop" onClick={() => setFormMode(null)}>
            <form className="task-modal-card user-admin-form" onClick={(e) => e.stopPropagation()} onSubmit={handleSaveUser}>
              <h3>{formMode.kind === 'edit' ? 'Editar usuario' : 'Nuevo usuario'}</h3>

              <label className="task-form-field">
                <span>Usuario *</span>
                <input value={username} onChange={(e) => setUsername(e.target.value)} disabled={saving} />
              </label>

              <label className="task-form-field">
                <span>
                  {formMode.kind === 'edit'
                    ? 'Nueva contraseña (vacío = no cambiar)'
                    : 'Contraseña *'}
                </span>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  disabled={saving}
                />
              </label>

              <label className="auth-switch-row">
                <span>Administrador</span>
                <input type="checkbox" checked={isAdmin} onChange={(e) => setIsAdmin(e.target.checked)} />
              </label>

              {formMode.kind === 'edit' ? (
                <label className="auth-switch-row">
                  <span>Cuenta activa</span>
                  <input type="checkbox" checked={isActive} onChange={(e) => setIsActive(e.target.checked)} />
                </label>
              ) : null}

              <div className="task-form-actions">
                <button type="button" className="btn-secondary" onClick={() => setFormMode(null)}>
                  Cancelar
                </button>
                <button type="submit" className="btn-primary" disabled={saving}>
                  {saving ? 'Guardando…' : 'Guardar'}
                </button>
              </div>
            </form>
          </div>
        ) : null}
      </div>
    </div>
  );
}
