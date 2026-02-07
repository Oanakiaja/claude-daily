import { useLanguage } from '../contexts/LanguageContext'
import { cn } from '../lib/utils'

export function LanguageToggle({ className }: { className?: string }) {
  const { language, setLanguage } = useLanguage()

  return (
    <button
      onClick={() => setLanguage(language === 'en' ? 'zh' : 'en')}
      className={cn(
        'px-2 py-1 rounded-lg text-sm font-medium transition-colors',
        'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200',
        'hover:bg-gray-100 dark:hover:bg-gray-800',
        className
      )}
    >
      {language === 'en' ? '中' : 'EN'}
    </button>
  )
}
