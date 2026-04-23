import { AbilityConstant, UIAbility, Want } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { window } from '@kit.ArkUI';
import bridge from 'libentry.so';

const DOMAIN = 0x3433;

export default class EntryAbility extends UIAbility {
  private mainWindow?: window.Window;

  private readonly onSystemDensityChange = (density: number): void => {
    bridge.setScaleFactor(this.normalizeDensityScale(density));
  };

  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(DOMAIN, 'cargo-ohos-app', '%{public}s', 'Ability onCreate');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    try {
      const mainWindow = windowStage.getMainWindowSync();
      this.mainWindow = mainWindow;
      this.syncDensityScale(mainWindow);
      mainWindow.on('systemDensityChange', this.onSystemDensityChange);
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

  onWindowStageDestroy(): void {
    if (this.mainWindow !== undefined) {
      try {
        this.mainWindow.off('systemDensityChange', this.onSystemDensityChange);
      } catch (err) {
        hilog.error(
          DOMAIN,
          'cargo-ohos-app',
          'Failed to unregister density listener: %{public}s',
          JSON.stringify(err)
        );
      }
      this.mainWindow = undefined;
    }
  }

  private syncDensityScale(mainWindow: window.Window): void {
    try {
      const densityInfo = mainWindow.getWindowDensityInfo();
      bridge.setScaleFactor(
        this.normalizeDensityScale(
          densityInfo.customDensity > 0
            ? densityInfo.customDensity
            : (densityInfo.systemDensity > 0 ? densityInfo.systemDensity : densityInfo.defaultDensity)
        )
      );
    } catch (err) {
      hilog.error(
        DOMAIN,
        'cargo-ohos-app',
        'Failed to read window density: %{public}s',
        JSON.stringify(err)
      );
      bridge.setScaleFactor(1.0);
    }
  }

  private normalizeDensityScale(density: number): number {
    if (typeof density === 'number' && Number.isFinite(density) && density > 0) {
      return density;
    }
    return 1.0;
  }
}
