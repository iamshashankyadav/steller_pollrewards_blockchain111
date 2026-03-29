"use client";

import { useState, useCallback, useEffect } from "react";
import {
  createPoll,
  castVote,
  closePoll,
  getPoll,
  getPollStats,
  getVoteRecord,
  CONTRACT_ADDRESS,
} from "@/hooks/contract";
import { AnimatedCard } from "@/components/ui/animated-card";
import { Spotlight } from "@/components/ui/spotlight";
import { ShimmerButton } from "@/components/ui/shimmer-button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// Types
interface Poll {
  poll_id: number;
  title: string;
  option_a: string;
  option_b: string;
  votes_a: number;
  votes_b: number;
  reward_per_vote: number;
  is_active: boolean;
  created_at: number;
}

interface PollStats {
  total_polls: number;
  active_polls: number;
  total_votes: number;
  total_rewarded: number;
}

interface VoteRecord {
  poll_id: number;
  voter_id: number;
  choice: number;
  rewarded: number;
  voted_at: number;
}

// ── Icons ────────────────────────────────────────────────────

function SpinnerIcon() {
  return (
    <svg className="animate-spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </svg>
  );
}

function VoteIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M9 12l2 2 4-4" />
      <path d="M21 12c-1 0-3-1-3-3s2-3 3-3 3 1 3 3-2 3-3 3" />
      <path d="M3 12c1 0 3-1 3-3s-2-3-3-3-3 1-3 3 2 3 3 3" />
      <path d="M12 3v18" />
    </svg>
  );
}

function TrophyIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6" />
      <path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18" />
      <path d="M4 22h16" />
      <path d="M10 14.66V17c0 .55-.47.995-.99 1.147a3.51 3.51 0 0 1-2.02 0C6.47 17.995 6 17.55 6 17v-2.34c0-.98.8-1.8 1.8-1.8h1.4c1 0 1.8.82 1.8 1.8Z" />
      <path d="m6 5 1.5 1.5" />
      <path d="m18 5-1.5 1.5" />
      <path d="M12 7v5" />
    </svg>
  );
}

function BarChartIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="20" x2="12" y2="10" />
      <line x1="18" y1="20" x2="18" y2="4" />
      <line x1="6" y1="20" x2="6" y2="16" />
    </svg>
  );
}

function PlusIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 5v14" />
      <path d="M5 12h14" />
    </svg>
  );
}

function XIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 6L6 18" />
      <path d="M6 6l12 12" />
    </svg>
  );
}

// ── Styled Input ─────────────────────────────────────────────

function Input({
  label,
  ...props
}: { label: string } & React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <div className="space-y-2">
      <label className="block text-[11px] font-medium uppercase tracking-wider text-white/30">
        {label}
      </label>
      <div className="group rounded-xl border border-white/[0.06] bg-white/[0.02] p-px transition-all focus-within:border-[#7c6cf0]/30 focus-within:shadow-[0_0_20px_rgba(124,108,240,0.08)]">
        <input
          {...props}
          className="w-full rounded-[11px] bg-transparent px-4 py-3 font-mono text-sm text-white/90 placeholder:text-white/15 outline-none"
        />
      </div>
    </div>
  );
}

// ── Method Signature ─────────────────────────────────────────

function MethodSignature({
  name,
  params,
  returns,
  color,
}: {
  name: string;
  params: string;
  returns?: string;
  color: string;
}) {
  return (
    <div className="flex items-center gap-2 rounded-xl border border-white/[0.04] bg-white/[0.02] px-4 py-3 font-mono text-sm">
      <span style={{ color }} className="font-semibold">fn</span>
      <span className="text-white/70">{name}</span>
      <span className="text-white/20 text-xs">{params}</span>
      {returns && (
        <span className="ml-auto text-white/15 text-[10px]">{returns}</span>
      )}
    </div>
  );
}

// ── Main Component ───────────────────────────────────────────

type Tab = "stats" | "create" | "vote" | "close";

interface ContractUIProps {
  walletAddress: string | null;
  onConnect: () => void;
  isConnecting: boolean;
}

export default function ContractUI({ walletAddress, onConnect, isConnecting }: ContractUIProps) {
  const [activeTab, setActiveTab] = useState<Tab>("stats");
  const [error, setError] = useState<string | null>(null);
  const [txStatus, setTxStatus] = useState<string | null>(null);

  // Create poll state
  const [pollTitle, setPollTitle] = useState("");
  const [optionA, setOptionA] = useState("");
  const [optionB, setOptionB] = useState("");
  const [rewardPerVote, setRewardPerVote] = useState("");
  const [isCreating, setIsCreating] = useState(false);

  // Vote state
  const [votePollId, setVotePollId] = useState("");
  const [voteChoice, setVoteChoice] = useState<"1" | "2">("1");
  const [isVoting, setIsVoting] = useState(false);

  // Close poll state
  const [closePollId, setClosePollId] = useState("");
  const [isClosing, setIsClosing] = useState(false);

  // Stats state
  const [stats, setStats] = useState<PollStats | null>(null);
  const [isLoadingStats, setIsLoadingStats] = useState(false);

  const truncate = (addr: string) => `${addr.slice(0, 6)}...${addr.slice(-4)}`;

  // Load stats on mount and when tab changes to stats
  useEffect(() => {
    if (activeTab === "stats") {
      loadStats();
    }
  }, [activeTab]);

  const loadStats = async () => {
    setIsLoadingStats(true);
    try {
      const pollStats = await getPollStats();
      setStats(pollStats);
    } catch (err) {
      console.error("Failed to load stats:", err);
    } finally {
      setIsLoadingStats(false);
    }
  };

  const handleCreatePoll = useCallback(async () => {
    if (!walletAddress) return setError("Connect wallet first");
    if (!pollTitle.trim() || !optionA.trim() || !optionB.trim() || !rewardPerVote) {
      return setError("Fill in all fields");
    }
    const reward = parseInt(rewardPerVote);
    if (isNaN(reward) || reward <= 0) return setError("Invalid reward amount");

    setError(null);
    setIsCreating(true);
    setTxStatus("Awaiting signature...");
    try {
      const result = await createPoll(walletAddress, pollTitle.trim(), optionA.trim(), optionB.trim(), reward);
      setTxStatus(`Poll created with ID: ${result.returnValue}`);
      setPollTitle("");
      setOptionA("");
      setOptionB("");
      setRewardPerVote("");
      setTimeout(() => setTxStatus(null), 5000);
    } catch (err: any) {
      setError(err.message || "Failed to create poll");
      setTxStatus(null);
    } finally {
      setIsCreating(false);
    }
  }, [walletAddress, pollTitle, optionA, optionB, rewardPerVote]);

  const handleCastVote = useCallback(async () => {
    if (!walletAddress) return setError("Connect wallet first");
    if (!votePollId) return setError("Enter poll ID");
    const pollId = parseInt(votePollId);
    if (isNaN(pollId) || pollId <= 0) return setError("Invalid poll ID");

    setError(null);
    setIsVoting(true);
    setTxStatus("Awaiting signature...");
    try {
      // Use wallet address hash as voter ID for simplicity
      const voterId = parseInt(walletAddress.slice(2, 10), 16);
      const result = await castVote(walletAddress, pollId, voterId, parseInt(voteChoice));
      setTxStatus(`Vote cast! Earned ${result.returnValue} reward tokens`);
      setVotePollId("");
      setTimeout(() => setTxStatus(null), 5000);
    } catch (err: any) {
      setError(err.message || "Failed to cast vote");
      setTxStatus(null);
    } finally {
      setIsVoting(false);
    }
  }, [walletAddress, votePollId, voteChoice]);

  const handleClosePoll = useCallback(async () => {
    if (!walletAddress) return setError("Connect wallet first");
    if (!closePollId) return setError("Enter poll ID");
    const pollId = parseInt(closePollId);
    if (isNaN(pollId) || pollId <= 0) return setError("Invalid poll ID");

    setError(null);
    setIsClosing(true);
    setTxStatus("Awaiting signature...");
    try {
      await closePoll(walletAddress, pollId);
      setTxStatus("Poll closed successfully!");
      setClosePollId("");
      setTimeout(() => setTxStatus(null), 5000);
    } catch (err: any) {
      setError(err.message || "Failed to close poll");
      setTxStatus(null);
    } finally {
      setIsClosing(false);
    }
  }, [walletAddress, closePollId]);

  return (
    <div className="w-full max-w-4xl mx-auto space-y-6">
      {/* Contract Address */}
      <div className="text-center">
        <p className="text-[10px] text-white/20 font-mono mb-1">Contract</p>
        <p className="text-xs text-white/40 font-mono">{truncate(CONTRACT_ADDRESS)}</p>
      </div>

      {/* Error Display */}
      {error && (
        <div className="rounded-xl border border-red-500/20 bg-red-500/5 p-4">
          <p className="text-sm text-red-400">{error}</p>
        </div>
      )}

      {/* Transaction Status */}
      {txStatus && (
        <div className="rounded-xl border border-green-500/20 bg-green-500/5 p-4">
          <p className="text-sm text-green-400">{txStatus}</p>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-1 rounded-xl border border-white/[0.06] bg-white/[0.02] p-1">
        {[
          { id: "stats" as Tab, label: "Statistics", icon: BarChartIcon },
          { id: "create" as Tab, label: "Create Poll", icon: PlusIcon },
          { id: "vote" as Tab, label: "Vote", icon: VoteIcon },
          { id: "close" as Tab, label: "Close Poll", icon: XIcon },
        ].map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveTab(id)}
            className={cn(
              "flex-1 flex items-center justify-center gap-2 rounded-lg px-4 py-2.5 text-sm font-medium transition-all",
              activeTab === id
                ? "bg-[#7c6cf0] text-white shadow-lg"
                : "text-white/40 hover:text-white/60"
            )}
          >
            <Icon />
            {label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <AnimatedCard className="p-6">
        <Spotlight className="from-[#7c6cf0]/20 via-transparent to-[#4fc3f7]/20" />

        {/* Statistics Tab */}
        {activeTab === "stats" && (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-lg font-semibold text-white mb-2">Poll Statistics</h3>
              <p className="text-sm text-white/40">Global poll metrics from the blockchain</p>
            </div>

            {isLoadingStats ? (
              <div className="flex justify-center py-8">
                <SpinnerIcon />
              </div>
            ) : stats ? (
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="text-center p-4 rounded-xl border border-white/[0.06] bg-white/[0.02]">
                  <p className="text-2xl font-bold text-[#7c6cf0]">{stats.total_polls}</p>
                  <p className="text-xs text-white/40 mt-1">Total Polls</p>
                </div>
                <div className="text-center p-4 rounded-xl border border-white/[0.06] bg-white/[0.02]">
                  <p className="text-2xl font-bold text-[#4fc3f7]">{stats.active_polls}</p>
                  <p className="text-xs text-white/40 mt-1">Active Polls</p>
                </div>
                <div className="text-center p-4 rounded-xl border border-white/[0.06] bg-white/[0.02]">
                  <p className="text-2xl font-bold text-[#34d399]">{stats.total_votes}</p>
                  <p className="text-xs text-white/40 mt-1">Total Votes</p>
                </div>
                <div className="text-center p-4 rounded-xl border border-white/[0.06] bg-white/[0.02]">
                  <p className="text-2xl font-bold text-[#fbbf24]">{stats.total_rewarded}</p>
                  <p className="text-xs text-white/40 mt-1">Rewards Distributed</p>
                </div>
              </div>
            ) : (
              <p className="text-center text-white/40">Failed to load statistics</p>
            )}

            <div className="pt-4 border-t border-white/[0.06]">
              <MethodSignature
                name="view_poll_stats"
                params="()"
                returns="PollStats"
                color="#4fc3f7"
              />
            </div>
          </div>
        )}

        {/* Create Poll Tab */}
        {activeTab === "create" && (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-lg font-semibold text-white mb-2">Create New Poll</h3>
              <p className="text-sm text-white/40">Set up a decentralized poll with automatic rewards</p>
            </div>

            <div className="space-y-4">
              <Input
                label="Poll Title"
                placeholder="What's the question?"
                value={pollTitle}
                onChange={(e) => setPollTitle(e.target.value)}
              />
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Input
                  label="Option A"
                  placeholder="First choice"
                  value={optionA}
                  onChange={(e) => setOptionA(e.target.value)}
                />
                <Input
                  label="Option B"
                  placeholder="Second choice"
                  value={optionB}
                  onChange={(e) => setOptionB(e.target.value)}
                />
              </div>
              <Input
                label="Reward Per Vote"
                type="number"
                placeholder="Tokens to reward each voter"
                value={rewardPerVote}
                onChange={(e) => setRewardPerVote(e.target.value)}
              />
            </div>

            <div className="flex gap-3">
              {!walletAddress ? (
                <ShimmerButton onClick={onConnect} disabled={isConnecting} className="flex-1">
                  {isConnecting ? <SpinnerIcon /> : "Connect Wallet"}
                </ShimmerButton>
              ) : (
                <ShimmerButton onClick={handleCreatePoll} disabled={isCreating} className="flex-1">
                  {isCreating ? <SpinnerIcon /> : <><PlusIcon /> Create Poll</>}
                </ShimmerButton>
              )}
            </div>

            <div className="pt-4 border-t border-white/[0.06]">
              <MethodSignature
                name="create_poll"
                params="(title, option_a, option_b, reward_per_vote)"
                returns="u64"
                color="#34d399"
              />
            </div>
          </div>
        )}

        {/* Vote Tab */}
        {activeTab === "vote" && (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-lg font-semibold text-white mb-2">Cast Your Vote</h3>
              <p className="text-sm text-white/40">Vote in an active poll and earn reward tokens</p>
            </div>

            <div className="space-y-4">
              <Input
                label="Poll ID"
                type="number"
                placeholder="Enter poll ID"
                value={votePollId}
                onChange={(e) => setVotePollId(e.target.value)}
              />
              <div className="space-y-2">
                <label className="block text-[11px] font-medium uppercase tracking-wider text-white/30">
                  Your Choice
                </label>
                <div className="flex gap-2">
                  <button
                    onClick={() => setVoteChoice("1")}
                    className={cn(
                      "flex-1 rounded-xl border p-4 text-sm font-medium transition-all",
                      voteChoice === "1"
                        ? "border-[#7c6cf0] bg-[#7c6cf0]/10 text-[#7c6cf0]"
                        : "border-white/[0.06] bg-white/[0.02] text-white/40 hover:text-white/60"
                    )}
                  >
                    Option A
                  </button>
                  <button
                    onClick={() => setVoteChoice("2")}
                    className={cn(
                      "flex-1 rounded-xl border p-4 text-sm font-medium transition-all",
                      voteChoice === "2"
                        ? "border-[#7c6cf0] bg-[#7c6cf0]/10 text-[#7c6cf0]"
                        : "border-white/[0.06] bg-white/[0.02] text-white/40 hover:text-white/60"
                    )}
                  >
                    Option B
                  </button>
                </div>
              </div>
            </div>

            <div className="flex gap-3">
              {!walletAddress ? (
                <ShimmerButton onClick={onConnect} disabled={isConnecting} className="flex-1">
                  {isConnecting ? <SpinnerIcon /> : "Connect Wallet"}
                </ShimmerButton>
              ) : (
                <ShimmerButton onClick={handleCastVote} disabled={isVoting} className="flex-1">
                  {isVoting ? <SpinnerIcon /> : <><VoteIcon /> Cast Vote</>}
                </ShimmerButton>
              )}
            </div>

            <div className="pt-4 border-t border-white/[0.06]">
              <MethodSignature
                name="cast_vote"
                params="(poll_id, voter_id, choice)"
                returns="u64"
                color="#fbbf24"
              />
            </div>
          </div>
        )}

        {/* Close Poll Tab */}
        {activeTab === "close" && (
          <div className="space-y-6">
            <div className="text-center">
              <h3 className="text-lg font-semibold text-white mb-2">Close Poll</h3>
              <p className="text-sm text-white/40">End voting for a poll (admin only)</p>
            </div>

            <div className="space-y-4">
              <Input
                label="Poll ID"
                type="number"
                placeholder="Enter poll ID to close"
                value={closePollId}
                onChange={(e) => setClosePollId(e.target.value)}
              />
            </div>

            <div className="flex gap-3">
              {!walletAddress ? (
                <ShimmerButton onClick={onConnect} disabled={isConnecting} className="flex-1">
                  {isConnecting ? <SpinnerIcon /> : "Connect Wallet"}
                </ShimmerButton>
              ) : (
                <ShimmerButton onClick={handleClosePoll} disabled={isClosing} className="flex-1">
                  {isClosing ? <SpinnerIcon /> : <><XIcon /> Close Poll</>}
                </ShimmerButton>
              )}
            </div>

            <div className="pt-4 border-t border-white/[0.06]">
              <MethodSignature
                name="close_poll"
                params="(poll_id)"
                color="#ef4444"
              />
            </div>
          </div>
        )}
      </AnimatedCard>
    </div>
  );
}
