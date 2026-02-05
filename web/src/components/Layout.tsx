import { useEffect, useCallback } from 'react'
import { NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom'
import { cn } from '../lib/utils'
import { ThemeToggle } from './ThemeToggle'

const TABS = ['/', '/jobs', '/settings'] as const

export function Layout() {
  const location = useLocation()
  const navigate = useNavigate()

  // Get current tab index based on pathname
  const getCurrentTabIndex = useCallback(() => {
    const path = location.pathname
    if (path === '/' || path.startsWith('/day')) return 0
    if (path === '/jobs') return 1
    if (path === '/settings') return 2
    return 0
  }, [location.pathname])

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Skip if user is typing in an input/textarea
      const target = e.target as HTMLElement
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        return
      }

      const currentIndex = getCurrentTabIndex()

      if (e.key === 'ArrowLeft') {
        e.preventDefault()
        const prevIndex = currentIndex > 0 ? currentIndex - 1 : TABS.length - 1
        navigate(TABS[prevIndex])
      } else if (e.key === 'ArrowRight') {
        e.preventDefault()
        const nextIndex = currentIndex < TABS.length - 1 ? currentIndex + 1 : 0
        navigate(TABS[nextIndex])
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [getCurrentTabIndex, navigate])

  return (
    <div className="min-h-dvh bg-white dark:bg-daily-dark text-gray-900 dark:text-gray-100 transition-colors">
      {/* Header */}
      <header className="fixed top-0 left-0 right-0 z-50 bg-white/90 dark:bg-daily-dark/90 backdrop-blur-md border-b border-gray-200 dark:border-orange-500/20 transition-colors">
        <div className="px-6 py-3 flex items-center gap-6">
          <NavLink to="/" className="text-xl font-bold text-orange-500 dark:text-orange-400 hover:text-orange-600 dark:hover:text-orange-300 transition-colors">
            Daily
          </NavLink>
          <nav className="flex gap-4 flex-1">
            <NavLink
              to="/"
              end
              className={({ isActive }) =>
                cn(
                  'text-sm transition-colors',
                  isActive
                    ? 'text-orange-500 dark:text-orange-400'
                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                )
              }
            >
              Archives
            </NavLink>
            <NavLink
              to="/jobs"
              className={({ isActive }) =>
                cn(
                  'text-sm transition-colors',
                  isActive
                    ? 'text-orange-500 dark:text-orange-400'
                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                )
              }
            >
              Jobs
            </NavLink>
            <NavLink
              to="/settings"
              className={({ isActive }) =>
                cn(
                  'text-sm transition-colors',
                  isActive
                    ? 'text-orange-500 dark:text-orange-400'
                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                )
              }
            >
              Settings
            </NavLink>
          </nav>
          <ThemeToggle />
        </div>
      </header>

      {/* Main Content */}
      <main className="pt-16 min-h-dvh">
        <Outlet />
      </main>
    </div>
  )
}
