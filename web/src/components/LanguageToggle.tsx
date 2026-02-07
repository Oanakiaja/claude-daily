import { useLanguage } from '../contexts/LanguageContext'
import { cn } from '../lib/utils'

export function LanguageToggle({ className }: { className?: string }) {
  const { language, setLanguage } = useLanguage()

  return (
    <button
      onClick={() => setLanguage(language === 'en' ? 'zh' : 'en')}
      className={cn(
        'p-2 rounded-lg transition-colors',
        'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200',
        'hover:bg-gray-100 dark:hover:bg-gray-800',
        className
      )}
      title={language === 'en' ? '切换到中文' : 'Switch to English'}
    >
      <span className="w-5 h-5 flex items-center justify-center text-sm font-medium leading-none">
        {language === 'en' ? 'EN' : '中'}
      </span>
    </button>
  )
}
