import { useState, useEffect, useRef, memo } from 'react'
import { motion } from 'framer-motion'
import { useApi } from '../hooks/useApi'
import type { ConversationMessage, ConversationContentBlock } from '../hooks/useApi'
import { MarkdownRenderer } from './MarkdownRenderer'
import { cn } from '../lib/utils'

interface ChatViewProps {
  date: string
  name: string
}

export function ChatView({ date, name }: ChatViewProps) {
  const [messages, setMessages] = useState<ConversationMessage[]>([])
  const [page, setPage] = useState(0)
  const [hasMore, setHasMore] = useState(false)
  const [hasTranscript, setHasTranscript] = useState(true)
  const [totalEntries, setTotalEntries] = useState(0)
  const [initialLoaded, setInitialLoaded] = useState(false)
  const { fetchConversation, loading } = useApi()
  const scrollRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    setMessages([])
    setPage(0)
    setInitialLoaded(false)
    loadPage(0)
  }, [date, name])

  const loadPage = async (p: number) => {
    try {
      const data = await fetchConversation(date, name, p)
      if (p === 0) {
        setMessages(data.messages)
      } else {
        setMessages(prev => [...prev, ...data.messages])
      }
      setPage(data.page)
      setHasMore(data.has_more)
      setHasTranscript(data.has_transcript)
      setTotalEntries(data.total_entries)
      setInitialLoaded(true)
    } catch (err) {
      console.error('Failed to load conversation:', err)
      setInitialLoaded(true)
    }
  }

  if (loading && !initialLoaded) {
    return (
      <div className="flex flex-col gap-4 p-4">
        {[1, 2, 3].map(i => (
          <div key={i} className={cn('flex', i % 2 === 0 ? 'justify-end' : 'justify-start')}>
            <div className="animate-pulse h-16 w-2/3 bg-gray-200 dark:bg-daily-dark rounded-lg" />
          </div>
        ))}
      </div>
    )
  }

  if (!hasTranscript && initialLoaded) {
    return (
      <div className="flex items-center justify-center py-16 text-gray-500 dark:text-gray-400">
        <div className="text-center">
          <svg className="size-12 mx-auto mb-3 opacity-40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
          <p className="text-sm">No transcript available for this session</p>
        </div>
      </div>
    )
  }

  if (messages.length === 0 && initialLoaded) {
    return (
      <div className="flex items-center justify-center py-16 text-gray-500 dark:text-gray-400">
        <p className="text-sm">No conversation messages found</p>
      </div>
    )
  }

  return (
    <div ref={scrollRef} className="flex flex-col gap-3 p-4">
      {messages.map((msg, i) => (
        <MessageBubble key={`${page}-${i}`} message={msg} />
      ))}
      {hasMore && (
        <div className="flex justify-center py-4">
          <button
            onClick={() => loadPage(page + 1)}
            disabled={loading}
            className={cn(
              'px-4 py-2 rounded-lg text-sm font-medium transition-colors',
              'bg-orange-500/10 text-orange-400 hover:bg-orange-500/20',
              'border border-orange-500/20 hover:border-orange-500/40',
              'disabled:opacity-50 disabled:cursor-not-allowed'
            )}
          >
            {loading ? 'Loading...' : `Load more (${totalEntries - messages.length} remaining)`}
          </button>
        </div>
      )}
    </div>
  )
}

const MessageBubble = memo(function MessageBubble({ message }: { message: ConversationMessage }) {
  const isUser = message.role === 'user'

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
      className={cn('flex', isUser ? 'justify-end' : 'justify-start')}
    >
      <div
        className={cn(
          'max-w-[85%] rounded-xl px-4 py-3',
          isUser
            ? 'bg-orange-500/15 border border-orange-500/25 rounded-br-sm'
            : 'bg-gray-100 dark:bg-daily-dark/80 border border-gray-200 dark:border-gray-700/50 rounded-bl-sm'
        )}
      >
        {/* Role label */}
        <div className={cn(
          'text-[11px] font-medium mb-1.5 flex items-center gap-2',
          isUser ? 'text-orange-400' : 'text-gray-400 dark:text-gray-500'
        )}>
          <span>{isUser ? 'You' : 'Claude'}</span>
          {message.timestamp && (
            <span className="text-gray-400 dark:text-gray-600 font-normal">
              {formatTimestamp(message.timestamp)}
            </span>
          )}
        </div>

        {/* Content blocks */}
        <div className="space-y-2">
          {message.content.map((block, i) => (
            <ContentBlockRenderer key={i} block={block} />
          ))}
        </div>
      </div>
    </motion.div>
  )
})

function ContentBlockRenderer({ block }: { block: ConversationContentBlock }) {
  switch (block.type) {
    case 'text':
      return (
        <div className="text-sm leading-relaxed text-gray-800 dark:text-gray-200">
          <MarkdownRenderer content={block.text} />
        </div>
      )
    case 'tool_use':
      return <ToolCallBlock name={block.name} input={block.input} toolUseId={block.tool_use_id} />
    case 'tool_result':
      return <ToolResultBlock content={block.content} />
    default:
      return null
  }
}

function ToolCallBlock({ name, input, toolUseId: _toolUseId }: { name: string; input: unknown; toolUseId: string }) {
  const [expanded, setExpanded] = useState(false)
  const summary = getToolSummary(name, input)

  return (
    <div className="my-1 border border-gray-300/50 dark:border-gray-600/50 rounded-md overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        className={cn(
          'w-full flex items-center gap-2 px-3 py-1.5 text-xs',
          'bg-gray-50/80 dark:bg-daily-dark/60 hover:bg-gray-100 dark:hover:bg-daily-dark',
          'text-gray-600 dark:text-gray-400 transition-colors text-left'
        )}
      >
        <span className="text-orange-500 font-semibold font-mono shrink-0">{name}</span>
        {summary && <span className="truncate font-mono opacity-70">{summary}</span>}
        <svg
          className={cn('size-3 shrink-0 ml-auto transition-transform', expanded && 'rotate-180')}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {expanded && (
        <div className="px-3 py-2 text-xs font-mono bg-gray-100/50 dark:bg-daily-dark border-t border-gray-300/50 dark:border-gray-600/50 max-h-60 overflow-auto">
          <pre className="whitespace-pre-wrap break-all text-gray-600 dark:text-gray-400">
            {typeof input === 'string' ? input : JSON.stringify(input, null, 2)}
          </pre>
        </div>
      )}
    </div>
  )
}

function ToolResultBlock({ content }: { content: string }) {
  const [expanded, setExpanded] = useState(false)

  if (!content) return null

  const preview = content.length > 100 ? content.slice(0, 100) + '...' : content

  return (
    <div className="my-1">
      <button
        onClick={() => setExpanded(!expanded)}
        className={cn(
          'w-full text-left px-3 py-1.5 text-xs font-mono rounded-md',
          'bg-green-500/5 dark:bg-green-500/5 text-gray-500 dark:text-gray-500',
          'border border-green-500/10 dark:border-green-500/10',
          'hover:bg-green-500/10 transition-colors'
        )}
      >
        <span className="text-green-600 dark:text-green-500 font-semibold mr-2">Result</span>
        {!expanded && <span className="opacity-60">{preview}</span>}
      </button>
      {expanded && (
        <div className="mt-1 px-3 py-2 text-xs font-mono bg-green-500/5 border border-green-500/10 rounded-md max-h-60 overflow-auto">
          <pre className="whitespace-pre-wrap break-all text-gray-600 dark:text-gray-400">
            {content}
          </pre>
        </div>
      )}
    </div>
  )
}

function getToolSummary(name: string, input: unknown): string {
  const inp = input as Record<string, unknown>
  if (!inp) return ''
  switch (name) {
    case 'Read':
      return String(inp.file_path || '')
    case 'Write':
      return String(inp.file_path || '')
    case 'Edit':
      return String(inp.file_path || '')
    case 'Bash':
      return String(inp.command || '').slice(0, 80)
    case 'Glob':
      return String(inp.pattern || '')
    case 'Grep':
      return String(inp.pattern || '')
    case 'Task':
      return String(inp.description || '').slice(0, 60)
    case 'WebFetch':
      return String(inp.url || '').slice(0, 60)
    case 'WebSearch':
      return String(inp.query || '').slice(0, 60)
    default:
      return ''
  }
}

function formatTimestamp(ts: string): string {
  try {
    const date = new Date(ts)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  } catch {
    return ts
  }
}
