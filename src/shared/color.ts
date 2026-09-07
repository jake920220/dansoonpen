export interface HSV { h: number; s: number; v: number }
export function hexToHsv(hex: string): HSV {
  const [r, g, b] = [1, 3, 5].map((offset) => parseInt(hex.slice(offset, offset + 2), 16) / 255);
  const max = Math.max(r, g, b), min = Math.min(r, g, b), delta = max - min;
  const hue = !delta ? 0 : max === r ? (g - b) / delta : max === g ? (b - r) / delta + 2 : (r - g) / delta + 4;
  return { h: (hue * 60 + 360) % 360, s: max ? delta / max : 0, v: max };
}
export function hsvToHex({ h, s, v }: HSV): string {
  h = ((h % 360) + 360) % 360; s = Math.max(0, Math.min(1, s)); v = Math.max(0, Math.min(1, v));
  const c = v * s, x = c * (1 - Math.abs((h / 60) % 2 - 1)), m = v - c;
  const rgb = h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x] : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
  return `#${rgb.map((n) => Math.round((n + m) * 255).toString(16).padStart(2, '0')).join('')}`;
}
export function wheelPosition(x: number, y: number): Pick<HSV, 'h' | 's'> {
  return { h: (Math.atan2(y, x) * 180 / Math.PI + 450) % 360, s: Math.min(1, Math.hypot(x, y)) };
}
