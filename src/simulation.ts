// Wave generator — reads UI controls and calls sim.splash()
// Does NOT own SimState — receives it from main.ts

import type { SimState } from '../pkg/ocean_wasm';

type WaveMode = 'off' | 'swell' | 'rain' | 'pulse';

export class WaveGenerator {
  private sim:  SimState;
  private grid: number;

  private waveTime:  number = 0;
  private rainTimer: number = 0;

  private modeSelect:  HTMLSelectElement;
  private ampSlider:   HTMLInputElement;
  private freqSlider:  HTMLInputElement;
  private speedSlider: HTMLInputElement;

  constructor(sim: SimState, grid: number) {
    this.sim  = sim;
    this.grid = grid;

    this.modeSelect  = this.getElement<HTMLSelectElement>('waveMode');
    this.ampSlider   = this.getElement<HTMLInputElement>('amp');
    this.freqSlider  = this.getElement<HTMLInputElement>('freq');
    this.speedSlider = this.getElement<HTMLInputElement>('speed');

    this.registerSliderListeners();
  }

  private getElement<T extends HTMLElement>(id: string): T {
    const el = document.getElementById(id) as T | null;
    if (!el) throw new Error(`Element #${id} not found`);
    return el;
  }

  private registerSliderListeners(): void {
    const bind = (slider: HTMLInputElement, displayId: string): void => {
      slider.addEventListener('input', () => {
        const display = document.getElementById(displayId);
        if (display) display.textContent = slider.value;
      });
    };

    bind(this.ampSlider,   'ampVal');
    bind(this.freqSlider,  'freqVal');
    bind(this.speedSlider, 'speedVal');
  }

  // Call once per frame before sim.step()
  update(): void {
    const mode  = this.modeSelect.value as WaveMode;
    const amp   = parseFloat(this.ampSlider.value);
    const freq  = parseFloat(this.freqSlider.value);
    const speed = parseFloat(this.speedSlider.value);

    switch (mode) {
      case 'swell':
        // Sine pattern along the top edge → waves roll inward
        for (let x = 0; x < this.grid; x++) {
          const wave = amp * Math.sin(freq * x + speed * this.waveTime);
          this.sim.splash(x, 0, wave * 0.05);
        }
        break;

      case 'rain': {
        // Random splashes at intervals controlled by speed
        this.rainTimer += speed * 0.05;
        if (this.rainTimer >= 1.0) {
          this.rainTimer = 0;
          const rx = Math.floor(Math.random() * this.grid);
          const rz = Math.floor(Math.random() * this.grid);
          this.sim.splash(rx, rz, amp * 0.5);
        }
        break;
      }

      case 'pulse': {
        // Periodic burst from center
        const cx = Math.floor(this.grid / 2);
        const cz = Math.floor(this.grid / 2);
        const pulse = amp * Math.max(0, Math.sin(speed * this.waveTime));
        if (pulse > 0.01) this.sim.splash(cx, cz, pulse * 0.03);
        break;
      }

      case 'off':
      default:
        break;
    }

    this.waveTime += 0.016;
  }

  // Manual splash — called on canvas click
  randomSplash(amount: number): void {
    const x = Math.floor(Math.random() * this.grid);
    const z = Math.floor(Math.random() * this.grid);
    this.sim.splash(x, z, amount);
  }
}
