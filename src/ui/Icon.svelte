<script lang="ts">
  let { name, size = 20 }: { name: string; size?: number } = $props();
  const paths: Record<string, string> = {
    eye: 'M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Zm13 0a3 3 0 1 1-6 0 3 3 0 0 1 6 0',
    grip: 'M9 5h.01M15 5h.01M9 12h.01M15 12h.01M9 19h.01M15 19h.01',
    collapse: 'm8 5 7 7-7 7',
    expand: 'm15 5-7 7 7 7',
    pen: 'm15 4 5 5M4 20l5-1L20 8a2.1 2.1 0 0 0-5-5L4 14l-1 7 7-2',
    text: 'M4 5h16M12 5v15M8 20h8M4 5v3M20 5v3',
    highlighter: 'm14 3 7 7-9 9-7-7 9-9ZM5 12l-2 7 2 2 7-2M3 21h7',
    eraser: 'm15 3 6 6-11 11H5l-3-3a2 2 0 0 1 0-3L13 3a1.5 1.5 0 0 1 2 0ZM7 9l8 8M10 20h11',
    undo: 'M8 4 3 9l5 5M3 9h11a6 6 0 0 1 0 12h-2',
    redo: 'm16 4 5 5-5 5M21 9H10a6 6 0 0 0 0 12h2',
    'clear-screen': 'M3 4h18v13H3zM8 21h8M12 17v4M9 7l6 7m-6 0 6-7',
    clear: 'M4 7h16M9 7V4h6v3M6 7l1 14h10l1-14M10 11v6M14 11v6',
    pointer: 'm5 3 14 10-7 1-3 7-4-18Z',
    settings: 'M4 6h16M4 12h16M4 18h16M8 3v6M16 9v6M10 15v6',
    monitor: 'M3 4h18v13H3zM8 21h8M12 17v4',
    arrow: 'M5 12h14m-5-5 5 5-5 5',
    check: 'm5 12 4 4L19 6',
    close: 'm6 6 12 12M6 18 18 6',
    info: 'M12 11v6M12 7h.01M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0',
    keyboard: 'M2 5h20v14H2zM6 9h.01M10 9h.01M14 9h.01M18 9h.01M6 13h.01M10 13h.01M14 13h.01M18 13h.01M7 16h10',
    power: 'M12 2v10M6 5a9 9 0 1 0 12 0',
  };
</script>
<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={paths[name] ?? paths.info} /></svg>
