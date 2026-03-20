import { reactive } from 'vue'
import type { AppSettings, AppView } from '../types'
import { DEFAULT_JUNK_SCAN_TARGET_IDS } from '../constants/junkScanTargets'

const STORAGE_KEY = 'macos-cleaner:settings'
const APP_VIEWS: AppView[] = ['dashboard', 'memory', 'junk', 'settings']

const DEFAULT_SETTINGS: AppSettings = {
  startupView: 'dashboard',
  showDashboardQuickActions: true,
  reduceMotion: false,
  autoSelectScanResults: true,
  confirmBeforeCleaning: true,
  enabledJunkTargets: [...DEFAULT_JUNK_SCAN_TARGET_IDS],
  rescanAfterCleaning: true,
  warningUsageThreshold: 70,
  criticalUsageThreshold: 85
}

const clamp = (value: number, min: number, max: number) => {
  return Math.min(Math.max(value, min), max)
}

const normalizeSettings = (value: Partial<AppSettings> | null | undefined): AppSettings => {
  const startupView = APP_VIEWS.includes(value?.startupView as AppView)
    ? (value?.startupView as AppView)
    : DEFAULT_SETTINGS.startupView

  const warningUsageThreshold = clamp(
    Number(value?.warningUsageThreshold ?? DEFAULT_SETTINGS.warningUsageThreshold) || DEFAULT_SETTINGS.warningUsageThreshold,
    50,
    90
  )

  const criticalUsageThreshold = clamp(
    Number(value?.criticalUsageThreshold ?? DEFAULT_SETTINGS.criticalUsageThreshold) || DEFAULT_SETTINGS.criticalUsageThreshold,
    warningUsageThreshold + 5,
    95
  )

  return {
    startupView,
    showDashboardQuickActions: value?.showDashboardQuickActions ?? DEFAULT_SETTINGS.showDashboardQuickActions,
    reduceMotion: value?.reduceMotion ?? DEFAULT_SETTINGS.reduceMotion,
    autoSelectScanResults: value?.autoSelectScanResults ?? DEFAULT_SETTINGS.autoSelectScanResults,
    confirmBeforeCleaning: value?.confirmBeforeCleaning ?? DEFAULT_SETTINGS.confirmBeforeCleaning,
    enabledJunkTargets:
      value?.enabledJunkTargets?.filter(targetId => DEFAULT_JUNK_SCAN_TARGET_IDS.includes(targetId))?.length
        ? value.enabledJunkTargets.filter(targetId => DEFAULT_JUNK_SCAN_TARGET_IDS.includes(targetId))
        : [...DEFAULT_SETTINGS.enabledJunkTargets],
    rescanAfterCleaning: value?.rescanAfterCleaning ?? DEFAULT_SETTINGS.rescanAfterCleaning,
    warningUsageThreshold,
    criticalUsageThreshold
  }
}

const loadSettings = (): AppSettings => {
  if (typeof window === 'undefined') {
    return { ...DEFAULT_SETTINGS }
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) {
      return { ...DEFAULT_SETTINGS }
    }

    return normalizeSettings(JSON.parse(raw) as Partial<AppSettings>)
  } catch (error) {
    console.warn('Failed to load settings:', error)
    return { ...DEFAULT_SETTINGS }
  }
}

const persistSettings = (settings: AppSettings) => {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
}

const settingsState = reactive<AppSettings>(loadSettings())

const applySettings = (nextSettings: AppSettings) => {
  Object.assign(settingsState, nextSettings)
  persistSettings(nextSettings)
}

export function useSettings() {
  const updateSettings = (patch: Partial<AppSettings>) => {
    applySettings(normalizeSettings({ ...settingsState, ...patch }))
  }

  const resetSettings = () => {
    applySettings({ ...DEFAULT_SETTINGS })
  }

  return {
    settings: settingsState,
    defaultSettings: DEFAULT_SETTINGS,
    updateSettings,
    resetSettings
  }
}
