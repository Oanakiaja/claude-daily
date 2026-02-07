import en from './en.json'
import zh from './zh.json'

export type Language = 'en' | 'zh'

export type TranslationKey = keyof typeof en

const translations: Record<Language, Record<string, string>> = { en, zh }

export function getTranslation(lang: Language, key: string, vars?: Record<string, string | number>): string {
  const value = translations[lang][key] ?? translations.en[key] ?? key
  if (!vars) return value
  return value.replace(/\{(\w+)\}/g, (_, k) => String(vars[k] ?? `{${k}}`))
}
