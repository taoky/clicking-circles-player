export class AudioService {
    private audio: HTMLAudioElement;
    private onEndCallback?: () => void;
    private onDurationChangeCallback?: (duration: number) => void;
    private onTimeUpdateCallback?: (time: number) => void;

    constructor() {
        this.audio = new Audio();
        this.audio.addEventListener('ended', () => {
            this.onEndCallback?.();
        });
        this.audio.addEventListener('durationchange', () => {
            if (this.audio.duration && this.audio.duration !== Infinity) {
                this.onDurationChangeCallback?.(this.audio.duration);
            }
        });
        this.audio.addEventListener('timeupdate', () => {
            this.onTimeUpdateCallback?.(this.audio.currentTime);
        });
    }

    load(url: string): Promise<void> {
        return new Promise((resolve, reject) => {
            this.audio.src = url;
            this.audio.load();
            
            const onCanPlay = () => {
                this.audio.removeEventListener('canplay', onCanPlay);
                this.audio.removeEventListener('error', onError);
                if (this.audio.duration && this.audio.duration !== Infinity) {
                    this.onDurationChangeCallback?.(this.audio.duration);
                }
                resolve();
            };

            const onError = (e: Event) => {
                this.audio.removeEventListener('canplay', onCanPlay);
                this.audio.removeEventListener('error', onError);
                reject(e);
            };

            this.audio.addEventListener('canplay', onCanPlay);
            this.audio.addEventListener('error', onError);
        });
    }

    play(): Promise<void> {
        return this.audio.play();
    }

    pause(): void {
        this.audio.pause();
    }

    stop(): void {
        this.audio.pause();
        this.audio.currentTime = 0;
    }

    seek(time: number): void {
        this.audio.currentTime = time;
    }

    getCurrentTime(): number {
        return this.audio.currentTime;
    }

    getDuration(): number {
        return this.audio.duration;
    }

    isPlaying(): boolean {
        return !this.audio.paused;
    }

    onEnd(callback: () => void): void {
        this.onEndCallback = callback;
    }

    onDurationChange(callback: (duration: number) => void): void {
        this.onDurationChangeCallback = callback;
    }

    onTimeUpdate(callback: (time: number) => void): void {
        this.onTimeUpdateCallback = callback;
    }

    destroy(): void {
        this.audio.pause();
        this.audio.src = '';
        this.audio.removeEventListener('ended', this.onEndCallback!);
        // Clean up all event listeners
        this.onEndCallback = undefined;
        this.onDurationChangeCallback = undefined;
        this.onTimeUpdateCallback = undefined;
    }
} 