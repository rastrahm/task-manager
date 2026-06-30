/**
 * Modal de administración de usuarios (solo administrador).
 * @module UserAdminModal
 */

import React, { useCallback, useEffect, useState } from 'react';
import { apiClient } from './apiClient';
import './Auth.css';

/**
 * @param {object} props
 * @param {boolean} props.open - Si el modal está visible.
 * @param {Function} props.onClose
 * @returns {JSX.Element|null}
 */
export default function UserAdminModal({ open, onClose }) {