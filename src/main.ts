import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';

document.body.dataset.view = new URLSearchParams(location.search).get('view') ?? 'control';
mount(App, { target: document.getElementById('app')! });
