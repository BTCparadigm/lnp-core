// LNP/BP Core Library implementing LNPBP specifications & standards
// Written in 2020-2022 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::fmt::Debug;
use std::ops::Range;

#[cfg(feature = "serde")]
use amplify::ToYamlString;
use lnp2p::legacy::{AcceptChannel, ChannelType, OpenChannel};

/// Limit for the maximum number of the accepted HTLCs towards some node
pub const BOLT3_MAX_ACCEPTED_HTLC_LIMIT: u16 = 483;

/// BOLT-3 dust limit
pub const BOLT3_DUST_LIMIT: u64 = 354;

/// Errors from [BOLT-2] policy validations for `open_channel` and
/// `accept_channel` messages.
///
/// [BOLT-2]: https://github.com/lightningnetwork/lightning-rfc/blob/master/02-peer-protocol.md
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Display,
    Error,
    StrictEncode,
    StrictDecode
)]
#[display(doc_comments)]
pub enum PolicyError {
    /// proposed `to_self_delay` value {proposed} is unreasonably large and
    /// exceeds node policy limit of {allowed_maximum}; rejecting the channel
    /// according to BOLT-2
    ToSelfDelayUnreasonablyLarge { proposed: u16, allowed_maximum: u16 },

    /// proposed limit for maximum accepted number of HTLCs {0} exceeds BOLT-3
    /// requirement to be below 483; rejecting the channel according to BOLT-2
    MaxAcceptedHtlcLimitExceeded(u16),

    /// proposed fee rate {proposed} sat/kw is outside of the fee rate policy
    /// of the local node ({lowest_accepted}..{highest_accepted} sat/kw);
    /// rejecting the channel according to BOLT-2
    FeeRateUnreasonable {
        proposed: u32,
        lowest_accepted: u32,
        highest_accepted: u32,
    },

    /// proposed channel reserve limit {reserve} sat is less than dust limit
    /// {dust_limit} sat; rejecting the channel according to BOLT-2
    ChannelReserveLessDust { reserve: u64, dust_limit: u64 },

    /// dust limit {0} sat is less than protocol minimum requirement of 354
    /// sat; rejecting the channel according to BOLT-2
    DustLimitTooSmall(u64),

    /// offered channel funding of {proposed} sat is too small and less than
    /// {required_minimum} required by the node policy; rejecting the channel
    /// according to BOLT-2
    ChannelFundingTooSmall {
        proposed: u64,
        required_minimum: u64,
    },

    /// HTLC minimum {proposed} is too large and exceeds node policy
    /// requirements ({allowed_maximum}); rejecting the channel according to
    /// BOLT-2
    HtlcMinimumTooLarge { proposed: u64, allowed_maximum: u64 },

    /// HTLC-in-flight maximum requirement of {proposed} is too small and
    /// does not match the node policy; the smallest requirement is
    /// {required_minimum}; rejecting the channel according to BOLT-2
    HtlcInFlightMaximumTooSmall {
        proposed: u64,
        required_minimum: u64,
    },

    /// requested {proposed} channel reserve is too large and exceeds local
    /// policy requirement of {allowed_maximum}; rejecting the channel
    /// according to BOLT-2
    ChannelReserveTooLarge { proposed: u64, allowed_maximum: u64 },

    /// maximum number of HTLCs {proposed} that can be accepted by the remote
    /// node is too small and does not match node policy requirement of
    /// {required_minimum}; rejecting the channel according to BOLT-2
    MaxAcceptedHtlcsTooSmall {
        proposed: u16,
        required_minimum: u16,
    },

    /// dust limit {proposed} sats exceeds node policy requirement of
    /// {allowed_maximum}; rejecting the channel according to BOLT-2
    DustLimitTooLarge { proposed: u64, allowed_maximum: u64 },

    /// minimum depth of {proposed} requested by the remote peer is exceeds
    /// local policy limit of {allowed_maximum}; rejecting the channel
    /// according to BOLT-2
    UnreasonableMinDepth { proposed: u32, allowed_maximum: u32 },

    /// `channel_reserve_satoshis` ({channel_reserve}) is less than
    /// `dust_limit_satoshis` ({dust_limit}) within the `open_channel`
    /// message; rejecting the channel according to BOLT-2
    LocalDustExceedsRemoteReserve {
        channel_reserve: u64,
        dust_limit: u64,
    },

    /// `channel_reserve_satoshis` from the open_channel message
    /// ({channel_reserve}) is less than `dust_limit_satoshis`
    /// ({dust_limit}; rejecting the channel according to BOLT-2
    RemoteDustExceedsLocalReserve {
        channel_reserve: u64,
        dust_limit: u64,
    },
}

/// Policy to validate channel parameters proposed by a remote peer.
///
/// By default, [`Channel::new`] uses reasonable default policy created by
/// [`Policy::default()`] method. Channel creator may provide a custom policy by
/// using [`Channel::with`] method.
#[derive(Clone, Eq, PartialEq, Hash, Debug, StrictEncode, StrictDecode)]
#[cfg_attr(
    feature = "serde",
    derive(Display, Serialize, Deserialize),
    serde(crate = "serde_crate"),
    display(Policy::to_yaml_string)
)]
pub struct Policy {
    /// Reasonable limit to check value of `to_self_delay` required by a remote
    /// node, in blocks.
    pub to_self_delay_max: u16,

    /// Range of acceptable channel fees.
    pub feerate_per_kw_range: Range<u32>,

    /// Minimum funding transaction mining depth required from the remote node
    /// for a channel proposed by it.
    pub minimum_depth: u32,

    // The following are optional policies which may not be set by a local
    // node:
    /// Maximum funding transaction mining depth which may be required by a
    /// remote node for a channel opened by a local node.
    pub maximum_depth: Option<u32>,

    /// Minimum funding for a channel by this node.
    pub funding_satoshis_min: Option<u64>,

    /// The maximum acceptable limit on the value stored in a single HTLC.
    pub htlc_minimum_msat_max: Option<u64>,

    /// Minimum boundary for the upper limit of in-flight HTLC funds.
    pub max_htlc_value_in_flight_msat_min: Option<u64>,

    /// Maximum reserve for a channel from a local node required by the remote
    /// node in absolute value.
    pub channel_reserve_satoshis_max_abs: Option<u64>,

    /// Maximum reserve for a channel from a local node required by the remote
    /// node in persents from the channel funding.
    pub channel_reserve_satoshis_max_percent: Option<u8>,

    /// Minimum boundary to the limit of HTLCs offered to a remote peer.
    pub max_accepted_htlcs_min: Option<u16>,

    /// Maximum value for the dust limit required by a remote node.
    pub dust_limit_satoshis_max: Option<u64>,
}

#[cfg(feature = "serde")]
impl ToYamlString for Policy {}

impl Default for Policy {
    /// Sets reasonable values for the local node policies
    fn default() -> Policy {
        Policy {
            to_self_delay_max: 250,
            // normal operational range for the fees in bitcoin network - it
            // really never went above 100 to get tx mined within an hour or two
            feerate_per_kw_range: 1..100,
            // three blocks is enough to get sufficient security
            minimum_depth: 3,
            // 6 blocks is enough to provide the necessary security
            maximum_depth: Some(6),
            // no reason of spamming blockchain with channels < 10000 sats
            funding_satoshis_min: Some(10000),
            // HTLCs can be arbitrary small:
            htlc_minimum_msat_max: None,
            // we need to earn commissions on routing, so limiting HTLCs too
            // much does not make sense
            max_htlc_value_in_flight_msat_min: Some(10000),
            max_accepted_htlcs_min: Some(10),
            // we do not want to over-collateralize on our channels in regard to
            // the size of the channel: it should not exceed 10% of funds in the
            // channel.
            channel_reserve_satoshis_max_abs: None,
            channel_reserve_satoshis_max_percent: Some(10),
            // we do not want to require too large `to_local` / `to_remote`
            // outputs
            dust_limit_satoshis_max: Some(1000),
        }
    }
}

impl Policy {
    /// Sets policy to match default policy used in c-lightning
    pub fn with_clightning_defaults() -> Policy {
        Policy {
            to_self_delay_max: 14 * 24 * 6,
            feerate_per_kw_range: 1..1000,
            minimum_depth: 3,
            maximum_depth: Some(6),
            funding_satoshis_min: Some(10000),
            htlc_minimum_msat_max: None,
            max_htlc_value_in_flight_msat_min: Some(10000),
            max_accepted_htlcs_min: Some(30),
            channel_reserve_satoshis_max_abs: None,
            // c-lightning uses 10% of the channel funding as a reserve
            channel_reserve_satoshis_max_percent: Some(10),
            dust_limit_satoshis_max: Some(546),
        }
    }

    /// Sets policy to match default policy used in LND
    pub fn with_lnd_defaults() -> Policy {
        Policy {
            to_self_delay_max: 14 * 24 * 6,
            feerate_per_kw_range: 1..1000,
            minimum_depth: 3,
            maximum_depth: Some(6),
            funding_satoshis_min: Some(20000),
            htlc_minimum_msat_max: None,
            max_htlc_value_in_flight_msat_min: Some(10000),
            max_accepted_htlcs_min: Some(483),
            channel_reserve_satoshis_max_abs: None,
            // LND uses 1% of the channel funding as a reserve
            channel_reserve_satoshis_max_percent: Some(1),
            // Since LND 0.13.3 there is no default value for dust limit
            // DustLimitForSize retrieves the dust limit for a given pkscript
            // size 546 is the biggest value for p2pkh
            // https://github.com/lightningnetwork/lnd/pull/5781
            dust_limit_satoshis_max: Some(546),
        }
    }

    /// Sets policy to match default policy used in Eclair
    pub fn with_eclair_defaults() -> Policy {
        Policy {
            to_self_delay_max: 14 * 24 * 6,
            feerate_per_kw_range: 1..1000,
            minimum_depth: 3,
            maximum_depth: Some(6),
            funding_satoshis_min: Some(100000),
            htlc_minimum_msat_max: None,
            max_htlc_value_in_flight_msat_min: Some(5000000000),
            max_accepted_htlcs_min: Some(30),
            channel_reserve_satoshis_max_abs: None,
            // LND uses 5% of the channel funding as a reserve
            channel_reserve_satoshis_max_percent: Some(5),
            dust_limit_satoshis_max: Some(546),
        }
    }

    fn validate_peer_params(
        &self,
        params: PeerParams,
    ) -> Result<(), PolicyError> {
        // if `to_self_delay` is unreasonably large.
        if params.to_self_delay > self.to_self_delay_max {
            return Err(PolicyError::ToSelfDelayUnreasonablyLarge {
                proposed: params.to_self_delay,
                allowed_maximum: self.to_self_delay_max,
            });
        }

        // if `max_accepted_htlcs` is greater than 483.
        if params.max_accepted_htlcs > BOLT3_MAX_ACCEPTED_HTLC_LIMIT {
            return Err(PolicyError::MaxAcceptedHtlcLimitExceeded(
                params.max_accepted_htlcs,
            ));
        }

        // if `dust_limit_satoshis` is greater than `channel_reserve_satoshis`.
        if params.dust_limit_satoshis > params.channel_reserve_satoshis {
            return Err(PolicyError::ChannelReserveLessDust {
                reserve: params.channel_reserve_satoshis,
                dust_limit: params.dust_limit_satoshis,
            });
        }

        // if `dust_limit_satoshis` is smaller than 354 satoshis
        if params.dust_limit_satoshis < BOLT3_DUST_LIMIT {
            return Err(PolicyError::DustLimitTooSmall(
                params.dust_limit_satoshis,
            ));
        }

        // if we consider `htlc_minimum_msat` too large
        if let Some(limit) = self.htlc_minimum_msat_max {
            if params.htlc_minimum_msat > limit {
                return Err(PolicyError::HtlcMinimumTooLarge {
                    proposed: params.htlc_minimum_msat,
                    allowed_maximum: limit,
                });
            }
        }

        // if we consider `max_htlc_value_in_flight_msat` too small
        if let Some(limit) = self.max_htlc_value_in_flight_msat_min {
            if params.max_htlc_value_in_flight_msat < limit {
                return Err(PolicyError::HtlcInFlightMaximumTooSmall {
                    proposed: params.max_htlc_value_in_flight_msat,
                    required_minimum: limit,
                });
            }
        }

        // if we consider `channel_reserve_satoshis` too large - in both abosute
        // and relative values
        if let Some(limit) = self.channel_reserve_satoshis_max_abs {
            if params.channel_reserve_satoshis > limit {
                return Err(PolicyError::ChannelReserveTooLarge {
                    proposed: params.channel_reserve_satoshis,
                    allowed_maximum: limit,
                });
            }
        }

        // if we consider `max_accepted_htlcs` too small
        if let Some(limit) = self.max_accepted_htlcs_min {
            if params.max_accepted_htlcs < limit {
                return Err(PolicyError::MaxAcceptedHtlcsTooSmall {
                    proposed: params.max_accepted_htlcs,
                    required_minimum: limit,
                });
            }
        }

        // if we consider `dust_limit_satoshis` too large
        if let Some(limit) = self.dust_limit_satoshis_max {
            if params.dust_limit_satoshis > limit {
                return Err(PolicyError::DustLimitTooLarge {
                    proposed: params.dust_limit_satoshis,
                    allowed_maximum: limit,
                });
            }
        }

        Ok(())
    }

    /// Validates parameters proposed by remote peer in `open_channel` message
    /// against the policy
    ///
    /// # Arguments
    /// - `self`: local policy;
    /// - `open_channel`: BOLT-2 message received by the peer.
    ///
    /// # Returns
    /// [`PeerParams`] to use for constructing channel transactions which should
    /// be signed by the local node.
    pub fn validate_inbound(
        &self,
        open_channel: &OpenChannel,
    ) -> Result<PeerParams, PolicyError> {
        // if we consider `feerate_per_kw` too small for timely processing or
        // unreasonably large.
        if !self
            .feerate_per_kw_range
            .contains(&open_channel.feerate_per_kw)
        {
            return Err(PolicyError::FeeRateUnreasonable {
                proposed: open_channel.feerate_per_kw,
                lowest_accepted: self.feerate_per_kw_range.start,
                highest_accepted: self.feerate_per_kw_range.end,
            });
        }

        // if `funding_satoshis` is too small
        if let Some(limit) = self.funding_satoshis_min {
            if open_channel.funding_satoshis < limit {
                return Err(PolicyError::ChannelFundingTooSmall {
                    proposed: open_channel.funding_satoshis,
                    required_minimum: limit,
                });
            }
        }

        // if we consider `channel_reserve_satoshis` too large - in both abosute
        // and relative values
        if let Some(percents) = self.channel_reserve_satoshis_max_percent {
            let limit = open_channel.funding_satoshis * percents as u64;
            if open_channel.channel_reserve_satoshis > limit {
                return Err(PolicyError::ChannelReserveTooLarge {
                    proposed: open_channel.channel_reserve_satoshis,
                    allowed_maximum: limit,
                });
            }
        }

        let peer_params = PeerParams::from(open_channel);
        self.validate_peer_params(peer_params)?;
        Ok(peer_params)
    }

    /// Confirms that parameters which were asked by a remote node via
    /// `accept_channel` message are confirming our policy.
    ///
    /// # Arguments
    /// - `self`: local policy;
    /// - `params`: parameters proposed by the local node in `open_channel`
    ///   message;
    /// - `accept_channel`: BOLT-2 message received by the peer.
    ///
    /// # Returns
    /// [`PeerParams`] to use for constructing channel transactions which should
    /// be signed by the local node.
    pub fn confirm_outbound(
        &self,
        our_params: PeerParams,
        accept_channel: &AcceptChannel,
    ) -> Result<PeerParams, PolicyError> {
        // if `minimum_depth` is unreasonably large:
        //
        //     MAY reject the channel.
        if let Some(limit) = self.maximum_depth {
            if accept_channel.minimum_depth > limit {
                return Err(PolicyError::UnreasonableMinDepth {
                    proposed: accept_channel.minimum_depth,
                    allowed_maximum: limit,
                });
            }
        }

        // if `channel_reserve_satoshis` is less than `dust_limit_satoshis`
        // within the open_channel message:
        //
        //     MUST reject the channel.
        if accept_channel.channel_reserve_satoshis
            < our_params.dust_limit_satoshis
        {
            return Err(PolicyError::LocalDustExceedsRemoteReserve {
                channel_reserve: accept_channel.channel_reserve_satoshis,
                dust_limit: our_params.dust_limit_satoshis,
            });
        }

        // if `channel_reserve_satoshis` from the open_channel message is less
        // than `dust_limit_satoshis`:
        //
        //     MUST reject the channel.
        if our_params.channel_reserve_satoshis
            < accept_channel.dust_limit_satoshis
        {
            return Err(PolicyError::RemoteDustExceedsLocalReserve {
                channel_reserve: our_params.channel_reserve_satoshis,
                dust_limit: accept_channel.dust_limit_satoshis,
            });
        }

        let peer_params = PeerParams::from(accept_channel);
        self.validate_peer_params(peer_params)?;
        Ok(peer_params)
    }
}

/// Structure containing part of the channel configuration (and state, as it
/// contains adjustible fee) which must follow specific policies and be accepted
/// or validated basing on those policies and additional protocol-level
/// requirements.
///
/// This information applies for both channel peers and used in constructing
/// both sides of asymmetric transactions.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, StrictEncode, StrictDecode)]
#[cfg_attr(
    feature = "serde",
    derive(Display, Serialize, Deserialize),
    serde(crate = "serde_crate"),
    display(CommonParams::to_yaml_string)
)]
pub struct CommonParams {
    /// Minimum depth of the funding transaction before the channel is
    /// considered open
    pub minimum_depth: u32,

    /// indicates the initial fee rate in satoshi per 1000-weight (i.e. 1/4 the
    /// more normally-used 'satoshi per 1000 vbytes') that this side will pay
    /// for commitment and HTLC transactions, as described in BOLT #3 (this can
    /// be adjusted later with an update_fee message).
    pub feerate_per_kw: u32,

    /// The least-significant bit of `channel_flags`. Indicates whether the
    /// initiator of the funding flow wishes to advertise this channel publicly
    /// to the network, as detailed within BOLT #7.
    pub announce_channel: bool,

    /// Channel types are an explicit enumeration: for convenience of future
    /// definitions they reuse even feature bits, but they are not an arbitrary
    /// combination (they represent the persistent features which affect the
    /// channel operation).
    pub channel_type: ChannelType,
}

#[cfg(feature = "serde")]
impl ToYamlString for CommonParams {}

impl Default for CommonParams {
    /// Sets reasonable values for the common channel parameters used in
    /// constructing `open_channel` message.
    ///
    /// Usually this should not be used and instead [`Channel::with`] should be
    /// provided with custom channel parameters basing on the current state of
    /// the bitcoin mempool and hash rate.
    fn default() -> Self {
        CommonParams {
            minimum_depth: 3,
            feerate_per_kw: 256,
            announce_channel: true,
            channel_type: ChannelType::default(),
        }
    }
}

impl CommonParams {
    /// Extracts common parameters from the incoming `open_channel` message and
    /// local default requirement for the minimum depth.
    #[inline]
    pub fn with(open_channel: &OpenChannel, minimum_depth: u32) -> Self {
        CommonParams {
            minimum_depth,
            feerate_per_kw: open_channel.feerate_per_kw,
            announce_channel: open_channel.should_announce_channel(),
            channel_type: open_channel.channel_type.unwrap_or_default(),
        }
    }
}

/// Structure containing part of the channel state which must follow specific
/// policies and be accepted or validated basing on those policies and
/// additional protocol-level requirements.
///
/// This information applies for only to one of the peers and requested by the
/// other peer. It is used in constructing transactions which should be signed
/// by the node demanding this requirements.
///
/// Should be instantiated from the node
/// configuration persistent storage and/or command line parameters and provided
/// to the channel constructor via [`Channel::with`].
///
/// Later, when creating new channels, it should be copied from the local
/// channel defaults object and updated / checked against local policies upon
/// receiving `accept_channel` reply by setting [`Bolt3::remote_params`] to a
/// value returned from
/// [`Bolt3::local_params`][`.confirm_outbound`](Policy::confirm_outbound)
/// method.
///
/// Upon receiving `open_channel` message from the remote node must validate the
/// proposed parameters against local policy with
/// [`Bolt3::policy`][`.validate_inbound`](Policy::validate_inbound) method and
/// assign the return value to [`Bolt3::remote_params`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, StrictEncode, StrictDecode)]
#[cfg_attr(
    feature = "serde",
    derive(Display, Serialize, Deserialize),
    serde(crate = "serde_crate"),
    display(PeerParams::to_yaml_string)
)]
pub struct PeerParams {
    /// The threshold below which outputs on transactions broadcast by sender
    /// will be omitted
    pub dust_limit_satoshis: u64,

    /// The number of blocks which the counterparty will have to wait to claim
    /// on-chain funds if they broadcast a commitment transaction
    pub to_self_delay: u16,

    /// Indicates the smallest value HTLC this node will accept.
    pub htlc_minimum_msat: u64,

    /// The maximum inbound HTLC value in flight towards sender, in
    /// milli-satoshi
    pub max_htlc_value_in_flight_msat: u64,

    /// The minimum value unencumbered by HTLCs for the counterparty to keep in
    /// the channel
    pub channel_reserve_satoshis: u64,

    /// The maximum number of inbound HTLCs towards sender
    pub max_accepted_htlcs: u16,
}

#[cfg(feature = "serde")]
impl ToYamlString for PeerParams {}

impl Default for PeerParams {
    /// Sets reasonable values for the channel parameters requested from the
    /// other peer in sent `open_channel` or `accept_channel` messages.
    ///
    /// Usually this should not be used and instead [`Channel::with`] should be
    /// provided with custom channel parameters basing on the user preferences.
    fn default() -> Self {
        PeerParams {
            dust_limit_satoshis: BOLT3_DUST_LIMIT,
            to_self_delay: 3,
            htlc_minimum_msat: 1,
            max_htlc_value_in_flight_msat: 1_000_000_000,
            channel_reserve_satoshis: 10000,
            max_accepted_htlcs: BOLT3_MAX_ACCEPTED_HTLC_LIMIT,
        }
    }
}

impl From<&OpenChannel> for PeerParams {
    /// Extracts peer-specific parameters from the incoming `open_channel`
    /// message. These parameters are applied to the local node.
    #[inline]
    fn from(open_channel: &OpenChannel) -> Self {
        PeerParams {
            dust_limit_satoshis: open_channel.dust_limit_satoshis,
            to_self_delay: open_channel.to_self_delay,
            htlc_minimum_msat: open_channel.htlc_minimum_msat,
            max_htlc_value_in_flight_msat: open_channel
                .max_htlc_value_in_flight_msat,
            channel_reserve_satoshis: open_channel.channel_reserve_satoshis,
            max_accepted_htlcs: open_channel.max_accepted_htlcs,
        }
    }
}

impl From<&AcceptChannel> for PeerParams {
    /// Extracts peer-specific parameters from the incoming `accept_channel`
    /// message. These parameters are applied to the local node.
    #[inline]
    fn from(accept_channel: &AcceptChannel) -> Self {
        PeerParams {
            dust_limit_satoshis: accept_channel.dust_limit_satoshis,
            to_self_delay: accept_channel.to_self_delay,
            htlc_minimum_msat: accept_channel.htlc_minimum_msat,
            max_htlc_value_in_flight_msat: accept_channel
                .max_htlc_value_in_flight_msat,
            channel_reserve_satoshis: accept_channel.channel_reserve_satoshis,
            max_accepted_htlcs: accept_channel.max_accepted_htlcs,
        }
    }
}
