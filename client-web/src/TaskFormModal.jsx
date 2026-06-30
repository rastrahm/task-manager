/**
 * Modal para crear y editar tareas con metadatos.
 * @module TaskFormModal
 */

import React, { useEffect, useState } from 'react';
import {
  metadataToJson,
  parseMetadata,
  parseTagsInput,
  PRIORITIES,
  tagsToInput,
} from './metadata';

/**
 * Modo del formulario: creación (raíz o subtarea) o edición.
 * @typedef {object} TaskFormMode
 * @property {'create' | 'edit'} kind
 * @property {number | null} [parentId]
 * @property {string} [parentTitle]
 * @property {module:api.Task} [task]
 */

/**
 * Valores enviados al guardar el formulario.
 * @typedef {object} TaskFormValues
 * @property {string} title
 * @property {string|null} [description]
 * @property {module:metadata.TaskMetadata} metadata
 * @property {number|null} [parentId]
 * @property {module:api.Task} [task]
 */

/**
 * Modal con título, descripción, prioridad, fecha límite y etiquetas.
 * @param {object} props
 * @param {TaskFormMode|null} props.mode - `null` oculta el modal.
 * @param {Function} props.onClose
 * @param {Function} props.onSubmit
 * @returns {JSX.Element|null}
 */
export default function TaskFormModal({ mode, onClose, onSubmit }) {