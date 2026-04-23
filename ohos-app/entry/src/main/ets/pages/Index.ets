import common from '@ohos.app.ability.common';
import bridge from 'libentry.so';

const runtimeBridge = bridge ?? {
  getMessage: (): string => 'Native bridge is unavailable.',
  incrementCounter: (): number => 0
};

@Entry
@Component
struct Index {
  @State message: string = runtimeBridge.getMessage();
  @State counter: number = 0;

  aboutToAppear(): void {
    const hostContext = this.getUIContext().getHostContext() as common.UIAbilityContext | undefined;
    const fontScale = hostContext?.config?.fontSizeScale;
    bridge.setFontScale(
      typeof fontScale === 'number' && Number.isFinite(fontScale) && fontScale > 0 ? fontScale : 1.0
    );
  }

  build() {
    Column({ space: 16 }) {
      Text('Rust -> OHOS')
        .fontSize(28)
        .fontWeight(FontWeight.Bold)

      Text(this.message)
        .fontSize(20)

      Text('Counter: ' + this.counter)
        .fontSize(18)

      Button('Call Rust')
        .onClick(() => {
          this.counter = runtimeBridge.incrementCounter();
          this.message = runtimeBridge.getMessage();
        })
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
    .alignItems(HorizontalAlign.Center)
    .padding(24)
  }
}
