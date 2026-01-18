import { useState, useCallback } from 'react';
import { Scrollama, Step } from 'react-scrollama';
import { motion, AnimatePresence } from 'framer-motion';
import DataFlowVisualization from './components/DataFlowVisualization';
import { steps, claudeStructure } from './data/content';

function App() {
  const [currentStep, setCurrentStep] = useState(0);
  const [progress, setProgress] = useState(0);

  const onStepEnter = useCallback(({ data }) => {
    setCurrentStep(data);
  }, []);

  const onStepProgress = useCallback(({ progress }) => {
    setProgress(progress);
  }, []);

  const currentStepData = steps[currentStep];

  return (
    <div className="min-h-screen bg-[#0a0a0a]">
      {/* Header */}
      <header className="fixed top-0 left-0 right-0 z-50 bg-[#0a0a0a]/90 backdrop-blur-md border-b border-orange-500/20">
        <div className="max-w-7xl mx-auto px-6 py-3 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <img src="/logo.png" alt="Daily" className="h-10 w-auto" />
          </div>
          <div className="flex items-center gap-4">
            <a
              href="https://github.com/oanakiaja/claude-daily"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-orange-400 hover:text-orange-300 transition-colors"
            >
              GitHub
            </a>
            <a
              href="#install"
              className="px-4 py-2 bg-orange-500 text-black text-sm font-medium rounded-lg hover:bg-orange-400 transition-colors"
            >
              Get Started
            </a>
          </div>
        </div>
        {/* Progress bar */}
        <div className="h-1 bg-[#1a1a1a]">
          <motion.div
            className="h-full bg-gradient-to-r from-orange-600 to-orange-400"
            style={{ width: `${((currentStep + progress) / steps.length) * 100}%` }}
            transition={{ duration: 0.2 }}
          />
        </div>
      </header>

      {/* Hero Section */}
      <section className="min-h-screen flex items-center justify-center px-6 pt-16">
        <div className="max-w-4xl mx-auto text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
          >
            <h1 className="text-5xl md:text-6xl font-bold text-gray-100 mb-6">
              Auto Log
              <br />
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-orange-500 to-orange-300">
                Your Work · Life · Code
              </span>
            </h1>
            <p className="text-xl text-gray-400 mb-4 max-w-2xl mx-auto">
              Seamlessly record every Claude Code session.
              No extra steps, no manual notes.
              Just code - Daily captures your journey automatically.
            </p>
            <p className="text-lg text-gray-500 mb-8 max-w-2xl mx-auto">
              Too many sessions to remember? Automatically distill reusable workflows from your coding journey.
            </p>
            <div className="flex items-center justify-center gap-4 flex-wrap">
              <code className="px-4 py-3 bg-[#1a1a1a] text-orange-400 rounded-lg text-sm font-mono border border-orange-500/30">
                curl -fsSL https://raw.githubusercontent.com/oanakiaja/claude-daily/main/scripts/install.sh | bash
              </code>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Main Scrollytelling Section */}
      <section className="relative">
        <div className="flex flex-col md:flex-row">
          {/* Sticky Visualization */}
          <div className="hidden md:block md:w-1/2 h-screen sticky top-16 bg-[#0f0f0f] border-r border-orange-500/10">
            <AnimatePresence mode="wait">
              <motion.div
                key={currentStep}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.3 }}
                className="w-full h-full"
              >
                <DataFlowVisualization
                  step={currentStep}
                  vizState={currentStepData.vizState}
                />
              </motion.div>
            </AnimatePresence>
          </div>

          {/* Scrollable Content */}
          <div className="w-full md:w-1/2">
            {/* Mobile visualization - shows above each step */}
            <div className="md:hidden sticky top-16 z-10 bg-[#0f0f0f] h-64 border-b border-orange-500/10">
              <DataFlowVisualization
                step={currentStep}
                vizState={currentStepData.vizState}
              />
            </div>

            <Scrollama
              onStepEnter={onStepEnter}
              onStepProgress={onStepProgress}
              offset={0.4}
            >
              {steps.map((step, index) => (
                <Step data={index} key={step.id}>
                  <div
                    className={`min-h-screen flex items-center px-8 md:px-12 py-16 transition-opacity duration-500 ${
                      currentStep === index ? 'opacity-100' : 'opacity-30'
                    }`}
                  >
                    <div className="max-w-md">
                      <motion.div
                        initial={{ opacity: 0, y: 30 }}
                        animate={{
                          opacity: currentStep === index ? 1 : 0.3,
                          y: currentStep === index ? 0 : 30,
                        }}
                        transition={{ duration: 0.5 }}
                      >
                        <span className="inline-block px-3 py-1 bg-orange-500/20 text-orange-400 text-xs font-medium rounded-full mb-4 border border-orange-500/30">
                          Step {index + 1}
                        </span>
                        <h2 className="text-3xl md:text-4xl font-bold text-gray-100 mb-6">
                          {step.title}
                        </h2>
                        <p className="text-lg text-gray-300 mb-4 leading-relaxed">
                          {step.content}
                        </p>
                        <p className="text-sm text-gray-500 leading-relaxed border-l-2 border-orange-500/50 pl-4">
                          {step.detail}
                        </p>
                      </motion.div>
                    </div>
                  </div>
                </Step>
              ))}
            </Scrollama>
          </div>
        </div>
      </section>

      {/* ~/.claude Structure Section */}
      <section className="py-24 px-6 bg-[#0f0f0f]">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-3xl font-bold text-center text-gray-100 mb-4">~/.claude/ Structure</h2>
          <p className="text-center text-gray-500 mb-12">Daily integrates seamlessly into Claude Code's directory</p>
          <div className="bg-[#1a1a1a] rounded-xl p-8 font-mono text-sm border border-orange-500/20">
            {claudeStructure.map((item, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.05 }}
                className="flex items-center gap-2 py-1"
                style={{ paddingLeft: `${item.level * 20}px` }}
              >
                {item.type === 'folder' ? (
                  <span className="text-orange-400">📁</span>
                ) : (
                  <span className="text-orange-300">📄</span>
                )}
                <span className={item.type === 'folder' ? 'text-orange-400' : 'text-gray-400'}>
                  {item.path}
                </span>
                {item.desc && (
                  <span className="text-gray-600 text-xs ml-2">← {item.desc}</span>
                )}
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Commands Section */}
      <section className="py-24 px-6 bg-[#0a0a0a]">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-3xl font-bold text-center text-gray-100 mb-12">Commands</h2>
          <div className="grid md:grid-cols-2 gap-6">
            {[
              { cmd: 'daily init -i', desc: 'Interactive setup with directory and digest config' },
              { cmd: 'daily install', desc: 'Install Claude Code hooks and slash commands' },
              { cmd: 'daily view', desc: 'View today\'s archive (interactive date selection)' },
              { cmd: 'daily digest', desc: 'Consolidate sessions into daily summary' },
              { cmd: 'daily today', desc: 'Quick alias for today\'s archive' },
              { cmd: 'daily yest', desc: 'Quick alias for yesterday\'s archive' },
              { cmd: 'daily extract-skill', desc: 'Extract reusable skill from session' },
              { cmd: 'daily config --show', desc: 'Show current configuration' },
            ].map((item, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                transition={{ delay: index * 0.05 }}
                className="bg-[#1a1a1a] rounded-xl p-4 border border-orange-500/20 hover:border-orange-500/40 transition-colors"
              >
                <code className="text-orange-400 font-mono font-medium">{item.cmd}</code>
                <p className="text-sm text-gray-500 mt-2">{item.desc}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Installation Section */}
      <section id="install" className="py-24 px-6 bg-gradient-to-br from-orange-600 to-orange-700">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-3xl font-bold mb-6 text-black">Start Auto Logging</h2>
          <p className="text-orange-900 mb-8">One command to begin your seamless dev journal</p>

          <div className="bg-[#0a0a0a]/90 rounded-xl p-6 mb-8 font-mono text-left overflow-x-auto border border-orange-400/30">
            <div className="text-orange-400/70 mb-2"># One-line install</div>
            <div className="text-orange-300 whitespace-nowrap">
              curl -fsSL https://raw.githubusercontent.com/oanakiaja/claude-daily/main/scripts/install.sh | bash
            </div>
            <div className="text-orange-400/70 mt-4 mb-2"># Or build from source</div>
            <div className="text-orange-300">git clone https://github.com/oanakiaja/claude-daily.git</div>
            <div className="text-orange-300">cd claude-daily</div>
            <div className="text-orange-300">cargo install --path .</div>
          </div>

          <div className="flex justify-center gap-4 flex-wrap">
            <a
              href="https://github.com/oanakiaja/claude-daily"
              target="_blank"
              rel="noopener noreferrer"
              className="px-6 py-3 bg-[#0a0a0a] text-orange-400 font-medium rounded-lg hover:bg-[#1a1a1a] transition-colors border border-orange-400/30"
            >
              View on GitHub
            </a>
            <a
              href="https://github.com/oanakiaja/claude-daily#readme"
              target="_blank"
              rel="noopener noreferrer"
              className="px-6 py-3 bg-black/30 text-black font-medium rounded-lg hover:bg-black/40 transition-colors"
            >
              Read the Docs
            </a>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-8 px-6 bg-[#0a0a0a] text-center text-gray-500 text-sm border-t border-orange-500/10">
        <p>
          <a href="https://github.com/oanakiaja/claude-daily" className="text-orange-400 hover:text-orange-300">
            GitHub
          </a>
        </p>
      </footer>
    </div>
  );
}

export default App;
