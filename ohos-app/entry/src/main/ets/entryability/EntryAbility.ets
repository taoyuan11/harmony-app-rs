import { AbilityConstant, UIAbility, Want } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { window } from '@kit.ArkUI';

const DOMAIN = 0x0000;

export default class EntryAbility extends UIAbility {
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(DOMAIN, 'cargo-ohos-app', '%{public}s', 'Ability onCreate');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    try {
      const mainWindow = windowStage.getMainWindowSync();
      mainWindow.setWindowLayoutFullScreen(true)
        .then(() => mainWindow.setWindowSystemBarEnable([]))
        .catch((err: Error) => {
          hilog.error(
            DOMAIN,
            'cargo-ohos-app',
            'Failed to configure immersive window: %{public}s',
            JSON.stringify(err)
          );
        });
    } catch (err) {
      hilog.error(
        DOMAIN,
        'cargo-ohos-app',
        'Failed to get main window: %{public}s',
        JSON.stringify(err)
      );
    }

    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(DOMAIN, 'cargo-ohos-app', 'Failed to load page: %{public}s', JSON.stringify(err));
      }
    });
  }
}
