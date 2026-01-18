import { useState } from 'react';
import { motion } from 'framer-motion';
import { commands, slashCommands, quickStart, globalOptions } from '../data/docs';
import type { Command } from '../data/docs';

interface CommandCardProps {
  command: Command;
  isExpanded: boolean;
  onClick: () => void;
}

function CommandCard({ command, isExpanded, onClick }: CommandCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-[#1a1a1a] rounded-xl border border-orange-500/20 hover:border-orange-500/40 transition-colors overflow-hidden"
    >
      <button
        onClick={onClick}
        className="w-full p-4 text-left flex items-center justify-between"
      >
        <div>
          <code className="text-orange-400 font-mono font-bold text-lg">{command.name}</code>
          <p className="text-sm text-gray-400 mt-1">{command.description}</p>
        </div>
        <span className={`text-gray-500 transition-transform ${isExpanded ? 'rotate-180' : ''}`}>
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </span>
      </button>

      {isExpanded && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: 'auto' }}
          exit={{ opacity: 0, height: 0 }}
          className="px-4 pb-4 border-t border-orange-500/10"
        >
          {/* Usage */}
          <div className="mt-4">
            <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">Usage</h4>
            <code className="block bg-[#0f0f0f] text-orange-300 px-3 py-2 rounded-lg text-sm font-mono overflow-x-auto">
              {command.usage}
            </code>
          </div>

          {/* Options */}
          {command.options && command.options.length > 0 && (
            <div className="mt-4">
              <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">Options</h4>
              <div className="space-y-2">
                {command.options.map((opt, idx) => (
                  <div key={idx} className="flex gap-4 text-sm">
                    <code className="text-orange-400 font-mono whitespace-nowrap">{opt.flag}</code>
                    <span className="text-gray-400">{opt.desc}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Subcommands */}
          {command.subcommands && command.subcommands.length > 0 && (
            <div className="mt-4">
              <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">Subcommands</h4>
              <div className="space-y-2">
                {command.subcommands.map((sub, idx) => (
                  <div key={idx} className="flex flex-col sm:flex-row sm:gap-4 text-sm">
                    <code className="text-orange-400 font-mono whitespace-nowrap">{sub.cmd}</code>
                    <span className="text-gray-400">{sub.desc}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Examples */}
          {command.examples && command.examples.length > 0 && (
            <div className="mt-4">
              <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">Examples</h4>
              <div className="space-y-2">
                {command.examples.map((ex, idx) => (
                  <div key={idx} className="bg-[#0f0f0f] rounded-lg p-3">
                    <code className="text-orange-300 font-mono text-sm">{ex.cmd}</code>
                    <p className="text-gray-500 text-xs mt-1">{ex.desc}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </motion.div>
      )}
    </motion.div>
  );
}

export default function Docs() {
  const [expandedCommand, setExpandedCommand] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');

  const filteredCommands = commands.filter(cmd =>
    cmd.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    cmd.description.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="min-h-screen bg-[#0a0a0a] pt-20 pb-16">
      {/* Hero */}
      <section className="px-6 py-16">
        <div className="max-w-4xl mx-auto text-center">
          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-4xl md:text-5xl font-bold text-gray-100 mb-4"
          >
            Documentation
          </motion.h1>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="text-lg text-gray-400"
          >
            Complete reference for Daily CLI commands and features
          </motion.p>
        </div>
      </section>

      {/* Quick Start */}
      <section className="px-6 pb-16">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-2xl font-bold text-gray-100 mb-6">Quick Start</h2>
          <div className="grid md:grid-cols-2 gap-4">
            {quickStart.map((item, idx) => (
              <motion.div
                key={idx}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: idx * 0.1 }}
                className="bg-[#1a1a1a] rounded-xl p-4 border border-orange-500/20"
              >
                <div className="flex items-center gap-3 mb-2">
                  <span className="w-6 h-6 rounded-full bg-orange-500 text-black text-sm font-bold flex items-center justify-center">
                    {item.step}
                  </span>
                  <span className="text-gray-200 font-medium">{item.title}</span>
                </div>
                <code className="block bg-[#0f0f0f] text-orange-300 px-3 py-2 rounded-lg text-sm font-mono overflow-x-auto">
                  {item.cmd}
                </code>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Commands */}
      <section className="px-6 pb-16">
        <div className="max-w-4xl mx-auto">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
            <h2 className="text-2xl font-bold text-gray-100">Commands</h2>
            <div className="relative">
              <input
                type="text"
                placeholder="Search commands..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full sm:w-64 bg-[#1a1a1a] border border-orange-500/20 rounded-lg px-4 py-2 text-gray-200 placeholder-gray-500 focus:outline-none focus:border-orange-500/50"
              />
              <svg
                className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
          </div>

          {/* Global Options */}
          <div className="mb-6 bg-[#1a1a1a] rounded-xl p-4 border border-orange-500/20">
            <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">Global Options</h3>
            <div className="grid sm:grid-cols-2 gap-2">
              {globalOptions.map((opt, idx) => (
                <div key={idx} className="flex gap-3 text-sm">
                  <code className="text-orange-400 font-mono whitespace-nowrap">{opt.flag}</code>
                  <span className="text-gray-500">{opt.desc}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Command List */}
          <div className="space-y-4">
            {filteredCommands.map((command) => (
              <CommandCard
                key={command.name}
                command={command}
                isExpanded={expandedCommand === command.name}
                onClick={() => setExpandedCommand(expandedCommand === command.name ? null : command.name)}
              />
            ))}
          </div>

          {filteredCommands.length === 0 && (
            <div className="text-center py-12 text-gray-500">
              No commands found matching "{searchTerm}"
            </div>
          )}
        </div>
      </section>

      {/* Slash Commands */}
      <section className="px-6 pb-16">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-2xl font-bold text-gray-100 mb-6">Slash Commands</h2>
          <p className="text-gray-400 mb-6">
            Use these commands directly in Claude Code after running <code className="text-orange-400">daily install</code>
          </p>
          <div className="grid gap-4">
            {slashCommands.map((cmd, idx) => (
              <motion.div
                key={idx}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                transition={{ delay: idx * 0.1 }}
                className="bg-[#1a1a1a] rounded-xl p-4 border border-orange-500/20"
              >
                <code className="text-orange-400 font-mono font-bold">{cmd.name}</code>
                <p className="text-gray-400 text-sm mt-1">{cmd.description}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Configuration */}
      <section className="px-6 pb-16">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-2xl font-bold text-gray-100 mb-6">Configuration</h2>
          <div className="bg-[#1a1a1a] rounded-xl p-6 border border-orange-500/20">
            <p className="text-gray-400 mb-4">
              Daily stores configuration in <code className="text-orange-400">~/.config/daily/config.toml</code>
            </p>
            <div className="bg-[#0f0f0f] rounded-lg p-4 font-mono text-sm">
              <div className="text-gray-500"># Example config.toml</div>
              <div className="text-orange-300 mt-2">storage_path = "~/.claude/daily"</div>
              <div className="text-orange-300">digest_time = "06:00"</div>
              <div className="text-orange-300">auto_digest = true</div>
            </div>
          </div>
        </div>
      </section>

      {/* Archive Structure */}
      <section className="px-6 pb-16">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-2xl font-bold text-gray-100 mb-6">Archive Structure</h2>
          <div className="bg-[#1a1a1a] rounded-xl p-6 border border-orange-500/20 font-mono text-sm">
            <div className="space-y-1">
              <div className="text-orange-400">~/.claude/daily/</div>
              <div className="text-gray-400 pl-4">2024-01-15/</div>
              <div className="text-gray-500 pl-8">daily.md        <span className="text-gray-600"># Consolidated daily summary</span></div>
              <div className="text-gray-500 pl-8">my-project.md   <span className="text-gray-600"># Session archive (before digest)</span></div>
              <div className="text-gray-500 pl-8">another-task.md</div>
              <div className="text-gray-400 pl-4">jobs/</div>
              <div className="text-gray-500 pl-8">*.json          <span className="text-gray-600"># Background job metadata</span></div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
