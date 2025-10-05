use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;

    /// Tracks encrypted vote tallies for a poll.
    pub struct VoteStats {
        minus_three: u64,
        minus_two: u64,
        minus_one: u64,
        zero: u64,
        one: u64,
        two: u64,
        three: u64,
    }

    /// Represents a single encrypted vote as an integer from 0..6
    /// 0 => -3, 1 => -2, ..., 6 => 3
    pub struct UserVote {
        vote_idx: u8,
    }

    #[instruction]
    pub fn init_vote_stats(mxe: Mxe) -> Enc<Mxe, VoteStats> {
        let vote_stats = VoteStats {
            minus_three: 0,
            minus_two: 0,
            minus_one: 0,
            zero: 0,
            one: 0,
            two: 0,
            three: 0,
        };
        mxe.from_arcis(vote_stats)
    }

    #[instruction]
    pub fn vote(
        vote_ctxt: Enc<Shared, UserVote>,
        vote_stats_ctxt: Enc<Mxe, VoteStats>,
    ) -> Enc<Mxe, VoteStats> {
        let user_vote = vote_ctxt.to_arcis();
        let mut vote_stats = vote_stats_ctxt.to_arcis();

        // Convert vote_stats into array for easier manipulation
        let mut counts = [
            vote_stats.minus_three,
            vote_stats.minus_two,
            vote_stats.minus_one,
            vote_stats.zero,
            vote_stats.one,
            vote_stats.two,
            vote_stats.three,
        ];

        // Increment the appropriate counter
        counts[user_vote.vote_idx as usize] += 1;

        // Assign back to struct
        vote_stats.minus_three = counts[0];
        vote_stats.minus_two = counts[1];
        vote_stats.minus_one = counts[2];
        vote_stats.zero = counts[3];
        vote_stats.one = counts[4];
        vote_stats.two = counts[5];
        vote_stats.three = counts[6];

        vote_stats_ctxt.owner.from_arcis(vote_stats)
    }

    #[instruction]
    pub fn reveal_result(vote_stats_ctxt: Enc<Mxe, VoteStats>) -> i8 {
        let vote_stats = vote_stats_ctxt.to_arcis();

        let counts = [
            vote_stats.minus_three,
            vote_stats.minus_two,
            vote_stats.minus_one,
            vote_stats.zero,
            vote_stats.one,
            vote_stats.two,
            vote_stats.three,
        ];

        // Compute total votes
        let mut total = 0u64;
        for i in 0..7 {
            total += counts[i];
        }

        // Compute cumulative sum to find median index
        let mid = total / 2;
        let mut cumulative = 0u64;
        let mut median_idx = 0u64;

        for i in 0..7 {
            let cond = (cumulative <= mid) && (mid < cumulative + counts[i]);
            median_idx = median_idx + (cond as u64) * (i as u64);
            cumulative += counts[i];
        }

        // Convert index to vote value (-3..3)
        ((median_idx as i8) - 3).reveal()
    }
}
