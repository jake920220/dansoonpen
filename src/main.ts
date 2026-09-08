import { mount } from 'svelte';
import App from './App.svelte';
import { installDiagnostics } from './app/diagnostics';
import './app.css';

const stopDiagnostics = installDiagnostics();
if (import.meta.hot) import.meta.hot.dispose(stopDiagnostics);
document.body.dataset.view = new URLSearchParams(location.search).get('view') ?? 'control';
mount(App, { target: document.getElementById('app')! });
