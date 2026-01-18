import { motion } from 'framer-motion';

const DataFlowVisualization = ({ step, vizState }) => {
  const {
    showClaudeCode,
    showHooks,
    showCommands,
    showTranscript,
    showSummarizer,
    showLog,
    highlight,
    activeFlow,
  } = vizState;

  const isHighlighted = (id) => highlight === id || highlight === 'all' || highlight === 'claude-dir';
  const isFlowActive = (id) => activeFlow?.includes(id) || highlight === 'all';

  const boxVariants = {
    hidden: { opacity: 0, scale: 0.8 },
    visible: { opacity: 1, scale: 1 },
  };

  const flowVariants = {
    hidden: { pathLength: 0, opacity: 0 },
    visible: { pathLength: 1, opacity: 1 },
  };

  return (
    <div className="w-full h-full flex items-center justify-center p-4">
      <svg
        viewBox="0 0 650 520"
        className="w-full h-full max-w-2xl"
        style={{ filter: 'drop-shadow(0 4px 20px rgba(0,0,0,0.3))' }}
      >
        <defs>
          {/* Gradients - Black & Orange Theme */}
          <linearGradient id="claudeGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#f97316" />
            <stop offset="100%" stopColor="#ea580c" />
          </linearGradient>
          <linearGradient id="hooksGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#fb923c" />
            <stop offset="100%" stopColor="#f97316" />
          </linearGradient>
          <linearGradient id="commandsGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#fbbf24" />
            <stop offset="100%" stopColor="#f59e0b" />
          </linearGradient>
          <linearGradient id="summarizerGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#fdba74" />
            <stop offset="100%" stopColor="#fb923c" />
          </linearGradient>
          <linearGradient id="logGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#fed7aa" />
            <stop offset="100%" stopColor="#fdba74" />
          </linearGradient>
          <linearGradient id="transcriptGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#c2410c" />
            <stop offset="100%" stopColor="#9a3412" />
          </linearGradient>

          {/* Arrow marker */}
          <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#f97316" />
          </marker>

          {/* Glow filter */}
          <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="4" result="coloredBlur"/>
            <feMerge>
              <feMergeNode in="coloredBlur"/>
              <feMergeNode in="SourceGraphic"/>
            </feMerge>
          </filter>
        </defs>

        {/* Background grid - dark theme */}
        <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
          <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#1a1a1a" strokeWidth="0.5"/>
        </pattern>
        <rect width="650" height="520" fill="url(#grid)" opacity="0.8"/>

        {/* ~/.claude directory label */}
        {(showHooks || showCommands || showLog) && (
          <motion.g
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.4 }}
          >
            <rect x="350" y="15" width="280" height="25" rx="4" fill="#1a1a1a" stroke="#f97316" strokeWidth="1" strokeDasharray="4,2"/>
            <text x="490" y="32" textAnchor="middle" fill="#f97316" fontSize="12" fontFamily="monospace">
              ~/.claude/
            </text>
          </motion.g>
        )}

        {/* Flow Lines */}
        {/* Claude Code to Hooks */}
        <motion.path
          d="M 150 100 L 150 160"
          fill="none"
          stroke={isFlowActive('claude-to-hooks') ? '#f97316' : '#2a2a2a'}
          strokeWidth={isFlowActive('claude-to-hooks') ? 3 : 2}
          strokeDasharray={isFlowActive('claude-to-hooks') ? '5,5' : 'none'}
          className={isFlowActive('claude-to-hooks') ? 'flow-line' : ''}
          markerEnd="url(#arrowhead)"
          variants={flowVariants}
          initial="hidden"
          animate={showHooks ? 'visible' : 'hidden'}
          transition={{ duration: 0.5 }}
        />

        {/* Claude Code to Transcript */}
        <motion.path
          d="M 200 70 Q 280 70 280 160"
          fill="none"
          stroke={isFlowActive('claude-to-transcript') ? '#c2410c' : '#2a2a2a'}
          strokeWidth={isFlowActive('claude-to-transcript') ? 3 : 2}
          strokeDasharray={isFlowActive('claude-to-transcript') ? '5,5' : 'none'}
          className={isFlowActive('claude-to-transcript') ? 'flow-line' : ''}
          markerEnd="url(#arrowhead)"
          variants={flowVariants}
          initial="hidden"
          animate={showTranscript ? 'visible' : 'hidden'}
          transition={{ duration: 0.5, delay: 0.2 }}
        />

        {/* Transcript to Hooks (session end) */}
        <motion.path
          d="M 230 210 L 200 210"
          fill="none"
          stroke={isFlowActive('transcript-to-hooks') ? '#c2410c' : '#2a2a2a'}
          strokeWidth={isFlowActive('transcript-to-hooks') ? 3 : 2}
          strokeDasharray={isFlowActive('transcript-to-hooks') ? '5,5' : 'none'}
          className={isFlowActive('transcript-to-hooks') ? 'flow-line' : ''}
          markerEnd="url(#arrowhead)"
          variants={flowVariants}
          initial="hidden"
          animate={showTranscript && showHooks ? 'visible' : 'hidden'}
          transition={{ duration: 0.5 }}
        />

        {/* Hooks to Summarizer */}
        <motion.path
          d="M 150 265 L 150 320"
          fill="none"
          stroke={isFlowActive('hooks-to-summarizer') ? '#fdba74' : '#2a2a2a'}
          strokeWidth={isFlowActive('hooks-to-summarizer') ? 3 : 2}
          strokeDasharray={isFlowActive('hooks-to-summarizer') ? '5,5' : 'none'}
          className={isFlowActive('hooks-to-summarizer') ? 'flow-line' : ''}
          markerEnd="url(#arrowhead)"
          variants={flowVariants}
          initial="hidden"
          animate={showSummarizer ? 'visible' : 'hidden'}
          transition={{ duration: 0.5 }}
        />

        {/* Summarizer to Log */}
        <motion.path
          d="M 200 370 L 360 370"
          fill="none"
          stroke={isFlowActive('summarizer-to-log') ? '#fed7aa' : '#2a2a2a'}
          strokeWidth={isFlowActive('summarizer-to-log') ? 3 : 2}
          strokeDasharray={isFlowActive('summarizer-to-log') ? '5,5' : 'none'}
          className={isFlowActive('summarizer-to-log') ? 'flow-line' : ''}
          markerEnd="url(#arrowhead)"
          variants={flowVariants}
          initial="hidden"
          animate={showLog && showSummarizer ? 'visible' : 'hidden'}
          transition={{ duration: 0.5 }}
        />

        {/* Hooks to Log (session start) */}
        <motion.path
          d="M 200 190 Q 300 190 380 280"
          fill="none"
          stroke={isFlowActive('hooks-to-log') ? '#fed7aa' : '#2a2a2a'}
          strokeWidth={isFlowActive('hooks-to-log') ? 3 : 2}
          strokeDasharray={isFlowActive('hooks-to-log') ? '5,5' : 'none'}
          className={isFlowActive('hooks-to-log') ? 'flow-line' : ''}
          markerEnd="url(#arrowhead)"
          variants={flowVariants}
          initial="hidden"
          animate={showLog ? 'visible' : 'hidden'}
          transition={{ duration: 0.5 }}
        />

        {/* Claude Code Box */}
        <motion.g
          variants={boxVariants}
          initial="hidden"
          animate={showClaudeCode ? 'visible' : 'hidden'}
          transition={{ duration: 0.4 }}
        >
          <rect
            x="50" y="40" width="200" height="80" rx="12"
            fill="url(#claudeGrad)"
            className={isHighlighted('claude-code') ? 'pulse-glow' : ''}
            filter={isHighlighted('claude-code') ? 'url(#glow)' : 'none'}
          />
          <text x="150" y="75" textAnchor="middle" fill="#0a0a0a" fontWeight="600" fontSize="16">
            Claude Code
          </text>
          <text x="150" y="98" textAnchor="middle" fill="rgba(10,10,10,0.7)" fontSize="12">
            AI Assistant
          </text>
        </motion.g>

        {/* Hooks Box - ~/.claude/hooks/ */}
        <motion.g
          variants={boxVariants}
          initial="hidden"
          animate={showHooks ? 'visible' : 'hidden'}
          transition={{ duration: 0.4, delay: 0.1 }}
        >
          <rect
            x="50" y="160" width="200" height="105" rx="12"
            fill="url(#hooksGrad)"
            className={isHighlighted('hooks') || isHighlighted('session-start') || isHighlighted('session-end') ? 'pulse-glow' : ''}
            filter={isHighlighted('hooks') || isHighlighted('session-start') || isHighlighted('session-end') ? 'url(#glow)' : 'none'}
          />
          <text x="150" y="185" textAnchor="middle" fill="#0a0a0a" fontWeight="600" fontSize="14">
            hooks/
          </text>
          {/* Session Start */}
          <rect
            x="60" y="195" width="80" height="30" rx="6"
            fill={isHighlighted('session-start') ? 'rgba(10,10,10,0.3)' : 'rgba(10,10,10,0.15)'}
          />
          <text x="100" y="215" textAnchor="middle" fill="#0a0a0a" fontSize="10">
            SessionStart
          </text>
          {/* Session End */}
          <rect
            x="160" y="195" width="80" height="30" rx="6"
            fill={isHighlighted('session-end') ? 'rgba(10,10,10,0.3)' : 'rgba(10,10,10,0.15)'}
          />
          <text x="200" y="215" textAnchor="middle" fill="#0a0a0a" fontSize="10">
            SessionEnd
          </text>
          <text x="150" y="250" textAnchor="middle" fill="rgba(10,10,10,0.6)" fontSize="9" fontFamily="monospace">
            ~/.claude/hooks/
          </text>
        </motion.g>

        {/* Transcript Box */}
        <motion.g
          variants={boxVariants}
          initial="hidden"
          animate={showTranscript ? 'visible' : 'hidden'}
          transition={{ duration: 0.4, delay: 0.2 }}
        >
          <rect
            x="230" y="160" width="120" height="100" rx="12"
            fill="url(#transcriptGrad)"
            className={isHighlighted('transcript') ? 'pulse-glow' : ''}
            filter={isHighlighted('transcript') ? 'url(#glow)' : 'none'}
          />
          <text x="290" y="190" textAnchor="middle" fill="#fed7aa" fontWeight="600" fontSize="14">
            Transcript
          </text>
          {/* File icon */}
          <rect x="265" y="200" width="50" height="35" rx="4" fill="rgba(254,215,170,0.2)"/>
          <line x1="273" y1="210" x2="307" y2="210" stroke="rgba(254,215,170,0.5)" strokeWidth="2"/>
          <line x1="273" y1="218" x2="300" y2="218" stroke="rgba(254,215,170,0.5)" strokeWidth="2"/>
          <line x1="273" y1="226" x2="303" y2="226" stroke="rgba(254,215,170,0.5)" strokeWidth="2"/>
        </motion.g>

        {/* Summarizer Box */}
        <motion.g
          variants={boxVariants}
          initial="hidden"
          animate={showSummarizer ? 'visible' : 'hidden'}
          transition={{ duration: 0.4, delay: 0.3 }}
        >
          <rect
            x="50" y="320" width="200" height="100" rx="12"
            fill="url(#summarizerGrad)"
            className={isHighlighted('summarizer') || isHighlighted('summarizer-active') ? 'pulse-glow' : ''}
            filter={isHighlighted('summarizer') || isHighlighted('summarizer-active') ? 'url(#glow)' : 'none'}
          />
          <text x="150" y="355" textAnchor="middle" fill="#0a0a0a" fontWeight="600" fontSize="14">
            Summarizer
          </text>
          <text x="150" y="375" textAnchor="middle" fill="rgba(10,10,10,0.7)" fontSize="11">
            AI Processing
          </text>
          {/* Thinking animation when active */}
          {(isHighlighted('summarizer-active') || isFlowActive('summarizer-thinking')) && (
            <g>
              <motion.circle
                cx="120" cy="400" r="4" fill="#0a0a0a"
                animate={{ opacity: [0.3, 1, 0.3] }}
                transition={{ duration: 1, repeat: Infinity, delay: 0 }}
              />
              <motion.circle
                cx="150" cy="400" r="4" fill="#0a0a0a"
                animate={{ opacity: [0.3, 1, 0.3] }}
                transition={{ duration: 1, repeat: Infinity, delay: 0.3 }}
              />
              <motion.circle
                cx="180" cy="400" r="4" fill="#0a0a0a"
                animate={{ opacity: [0.3, 1, 0.3] }}
                transition={{ duration: 1, repeat: Infinity, delay: 0.6 }}
              />
            </g>
          )}
        </motion.g>

        {/* Log Box - ~/.claude/daily/ */}
        <motion.g
          variants={boxVariants}
          initial="hidden"
          animate={showLog ? 'visible' : 'hidden'}
          transition={{ duration: 0.4, delay: 0.4 }}
        >
          <rect
            x="360" y="280" width="160" height="175" rx="12"
            fill="url(#logGrad)"
            className={isHighlighted('session-file') || isHighlighted('daily-file') ? 'pulse-glow' : ''}
            filter={isHighlighted('session-file') || isHighlighted('daily-file') ? 'url(#glow)' : 'none'}
          />
          <text x="440" y="305" textAnchor="middle" fill="#0a0a0a" fontWeight="600" fontSize="14">
            daily/
          </text>
          <text x="440" y="320" textAnchor="middle" fill="rgba(10,10,10,0.6)" fontSize="9" fontFamily="monospace">
            ~/.claude/daily/
          </text>

          {/* Date folder */}
          <rect x="375" y="330" width="130" height="22" rx="4" fill="rgba(10,10,10,0.1)"/>
          <text x="385" y="346" fill="rgba(10,10,10,0.7)" fontSize="10" fontFamily="monospace">2024-01-15/</text>

          {/* File structure */}
          <rect
            x="385" y="358" width="110" height="22" rx="4"
            fill={isHighlighted('daily-file') ? 'rgba(10,10,10,0.25)' : 'rgba(10,10,10,0.1)'}
          />
          <text x="395" y="374" fill="#0a0a0a" fontSize="10" fontFamily="monospace">daily.md</text>

          <rect
            x="385" y="386" width="110" height="22" rx="4"
            fill={isHighlighted('session-file') ? 'rgba(10,10,10,0.25)' : 'rgba(10,10,10,0.1)'}
          />
          <text x="395" y="402" fill="#0a0a0a" fontSize="10" fontFamily="monospace">session.md</text>

          <rect x="375" y="414" width="130" height="22" rx="4" fill="rgba(10,10,10,0.08)"/>
          <text x="385" y="430" fill="rgba(10,10,10,0.6)" fontSize="10" fontFamily="monospace">jobs/</text>
        </motion.g>

        {/* Commands Box - ~/.claude/commands/ */}
        <motion.g
          variants={boxVariants}
          initial="hidden"
          animate={showCommands ? 'visible' : 'hidden'}
          transition={{ duration: 0.4, delay: 0.5 }}
        >
          <rect
            x="530" y="280" width="100" height="175" rx="12"
            fill="url(#commandsGrad)"
            className={isHighlighted('commands') ? 'pulse-glow' : ''}
            filter={isHighlighted('commands') ? 'url(#glow)' : 'none'}
          />
          <text x="580" y="305" textAnchor="middle" fill="#0a0a0a" fontWeight="600" fontSize="14">
            commands/
          </text>
          <text x="580" y="320" textAnchor="middle" fill="rgba(10,10,10,0.6)" fontSize="8" fontFamily="monospace">
            ~/.claude/commands/
          </text>

          {/* Command files */}
          <rect x="540" y="330" width="80" height="22" rx="4" fill="rgba(10,10,10,0.1)"/>
          <text x="548" y="346" fill="rgba(10,10,10,0.8)" fontSize="9" fontFamily="monospace">/daily-view</text>

          <rect x="540" y="358" width="80" height="22" rx="4" fill="rgba(10,10,10,0.1)"/>
          <text x="548" y="374" fill="rgba(10,10,10,0.8)" fontSize="8" fontFamily="monospace">/daily-get-skill</text>

          <rect x="540" y="386" width="80" height="22" rx="4" fill="rgba(10,10,10,0.1)"/>
          <text x="548" y="402" fill="rgba(10,10,10,0.8)" fontSize="8" fontFamily="monospace">/daily-get-cmd</text>
        </motion.g>

        {/* Commands flow from Claude Code */}
        {isFlowActive('commands-flow') && (
          <motion.path
            d="M 250 80 Q 400 50 580 275"
            fill="none"
            stroke="#fbbf24"
            strokeWidth="2"
            strokeDasharray="5,5"
            className="flow-line"
            markerEnd="url(#arrowhead)"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.5 }}
          />
        )}

        {/* Complete cycle animation */}
        {highlight === 'all' && (
          <motion.g>
            <motion.circle
              cx="320" cy="280" r="230"
              fill="none"
              stroke="url(#claudeGrad)"
              strokeWidth="2"
              strokeDasharray="10,10"
              initial={{ rotate: 0 }}
              animate={{ rotate: 360 }}
              transition={{ duration: 20, repeat: Infinity, ease: 'linear' }}
              style={{ transformOrigin: '320px 280px' }}
            />
          </motion.g>
        )}

        {/* Step indicator */}
        <text x="325" y="505" textAnchor="middle" fill="#6b7280" fontSize="12">
          Step {step + 1} of 12
        </text>
      </svg>
    </div>
  );
};

export default DataFlowVisualization;
