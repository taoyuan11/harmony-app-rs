declare const bridge: {
  getMessage(): string;
  incrementCounter(): number;
  setScaleFactor(scaleFactor: number): void;
  setFontScale(fontScale: number): void;
};

export default bridge;
