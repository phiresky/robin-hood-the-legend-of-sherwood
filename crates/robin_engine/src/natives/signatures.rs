//! Script-native signature metadata used by tooling.
//!
//! `NativeFn` remains the source of truth for native indices and names. This
//! table carries the extra return/parameter metadata needed by the decompiler
//! and debug HTTP endpoints.

use super::{NativeFn, native_name};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeParamSig {
    pub ty: &'static str,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSignature {
    pub name: &'static str,
    pub return_type: &'static str,
    pub params: &'static [NativeParamSig],
}

macro_rules! sig {
    ($name:literal, $return_type:literal, [$($param:expr),* $(,)?]) => {
        NativeSignature {
            name: $name,
            return_type: $return_type,
            params: &[$($param),*],
        }
    };
}

pub const NATIVE_SIGNATURES: &[NativeSignature] = &[
    sig!(
        "InitGlobal",
        "void",
        [
            NativeParamSig {
                ty: "int",
                name: "iID"
            },
            NativeParamSig {
                ty: "int",
                name: "iValue"
            }
        ]
    ),
    sig!(
        "SetGlobal",
        "void",
        [
            NativeParamSig {
                ty: "int",
                name: "iID"
            },
            NativeParamSig {
                ty: "int",
                name: "iValue"
            }
        ]
    ),
    sig!(
        "GetGlobal",
        "int",
        [NativeParamSig {
            ty: "int",
            name: "iID"
        }]
    ),
    sig!(
        "GetActorScript",
        "Actor",
        [NativeParamSig {
            ty: "int",
            name: "iPosition"
        }]
    ),
    sig!(
        "GetDoorScript",
        "Door",
        [NativeParamSig {
            ty: "int",
            name: "iPosition"
        }]
    ),
    sig!(
        "GetPatchScript",
        "Patch",
        [NativeParamSig {
            ty: "int",
            name: "iPosition"
        }]
    ),
    sig!(
        "GetLocationScript",
        "Location",
        [NativeParamSig {
            ty: "int",
            name: "iPosition"
        }]
    ),
    sig!(
        "GetSoundSourceScript",
        "SoundSource",
        [NativeParamSig {
            ty: "int",
            name: "iPosition"
        }]
    ),
    sig!(
        "GetBuildingScript",
        "Building",
        [NativeParamSig {
            ty: "int",
            name: "iPosition"
        }]
    ),
    sig!(
        "GetWayScript",
        "Way",
        [NativeParamSig {
            ty: "int",
            name: "iPosition"
        }]
    ),
    sig!(
        "GetActorIndex",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "GetDoorIndex",
        "int",
        [NativeParamSig {
            ty: "Door",
            name: "door"
        }]
    ),
    sig!(
        "GetPatchIndex",
        "int",
        [NativeParamSig {
            ty: "Patch",
            name: "patch"
        }]
    ),
    sig!(
        "GetLocationIndex",
        "int",
        [NativeParamSig {
            ty: "Location",
            name: "location"
        }]
    ),
    sig!(
        "GetSoundSourceIndex",
        "int",
        [NativeParamSig {
            ty: "SoundSource",
            name: "soundsource"
        }]
    ),
    sig!(
        "GetBuildingIndex",
        "int",
        [NativeParamSig {
            ty: "Building",
            name: "building"
        }]
    ),
    sig!(
        "GetWayIndex",
        "int",
        [NativeParamSig {
            ty: "Way",
            name: "way"
        }]
    ),
    sig!(
        "StartDialog",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iDialogue"
        }]
    ),
    sig!(
        "ScrollCameraTo",
        "bool",
        [NativeParamSig {
            ty: "Location",
            name: "location"
        }]
    ),
    sig!(
        "ScrollCameraSlowlyTo",
        "bool",
        [
            NativeParamSig {
                ty: "Location",
                name: "location"
            },
            NativeParamSig {
                ty: "float",
                name: "fSpeed"
            }
        ]
    ),
    sig!(
        "JumpCameraTo",
        "bool",
        [NativeParamSig {
            ty: "Location",
            name: "location"
        }]
    ),
    sig!(
        "SetZoomLevel",
        "bool",
        [NativeParamSig {
            ty: "float",
            name: "fZoom"
        }]
    ),
    sig!(
        "DisplayMap",
        "bool",
        [NativeParamSig {
            ty: "bool",
            name: "bDisplay"
        }]
    ),
    sig!("DisplayConsole", "void", []),
    sig!(
        "CustomizeMinimapDisplay",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iKindOfDot"
            }
        ]
    ),
    sig!(
        "DefineFlatTrajectoryZone",
        "void",
        [
            NativeParamSig {
                ty: "Location",
                name: "pLocation"
            },
            NativeParamSig {
                ty: "int",
                name: "iApex"
            }
        ]
    ),
    sig!(
        "AddShortBriefing",
        "void",
        [
            NativeParamSig {
                ty: "int",
                name: "iID"
            },
            NativeParamSig {
                ty: "bool",
                name: "bPrimary"
            }
        ]
    ),
    sig!(
        "DoneShortBriefing",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iID"
        }]
    ),
    sig!(
        "ChooseVictoryDefeatText",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iID"
        }]
    ),
    sig!("ForceCheckVictory", "void", []),
    sig!("Start", "bool", []),
    sig!("Thanx", "bool", []),
    sig!("Then", "int", []),
    sig!(
        "RecordScrollCameraTo",
        "bool",
        [NativeParamSig {
            ty: "Location",
            name: "location"
        }]
    ),
    sig!(
        "RecordJumpCameraTo",
        "bool",
        [NativeParamSig {
            ty: "Location",
            name: "location"
        }]
    ),
    sig!(
        "RecordSetZoom",
        "bool",
        [NativeParamSig {
            ty: "float",
            name: "fZoomLevel"
        }]
    ),
    sig!(
        "RecordDisplayMap",
        "bool",
        [NativeParamSig {
            ty: "bool",
            name: "bDisplay"
        }]
    ),
    sig!(
        "RecordActionAvailable",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iAction"
            },
            NativeParamSig {
                ty: "bool",
                name: "bAvailable"
            }
        ]
    ),
    sig!(
        "RecordCharacterAvailable",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "bool",
                name: "bAvailable"
            }
        ]
    ),
    sig!(
        "RecordLockCameraOn",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!("RecordClearCameraLock", "bool", []),
    sig!(
        "RecordPlayDialog",
        "bool",
        [NativeParamSig {
            ty: "int",
            name: "iDialogID"
        }]
    ),
    sig!(
        "RecordMoveCameraTo",
        "bool",
        [
            NativeParamSig {
                ty: "Location",
                name: "destination"
            },
            NativeParamSig {
                ty: "int",
                name: "iSpeed"
            }
        ]
    ),
    sig!(
        "RecordSendMessage",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actReceiver"
            },
            NativeParamSig {
                ty: "int",
                name: "iMessageCode"
            }
        ]
    ),
    sig!(
        "RecordSendMessageWithArguments",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actReceiver"
            },
            NativeParamSig {
                ty: "int",
                name: "iMessageCode"
            },
            NativeParamSig {
                ty: "int",
                name: "iArgument1"
            },
            NativeParamSig {
                ty: "int",
                name: "iArgument2"
            }
        ]
    ),
    sig!(
        "RecordMove",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "location"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            }
        ]
    ),
    sig!(
        "RecordEnterGame",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "location"
            },
            NativeParamSig {
                ty: "int",
                name: "iDirection"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            }
        ]
    ),
    sig!(
        "RecordLeaveGame",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "location"
            },
            NativeParamSig {
                ty: "int",
                name: "iDirection"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            }
        ]
    ),
    sig!(
        "RecordTurnTo",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "location"
            }
        ]
    ),
    sig!(
        "RecordPlayAnim",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iId"
            }
        ]
    ),
    sig!(
        "RecordPlayAnimLoop",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iId"
            }
        ]
    ),
    sig!(
        "RecordPlayAnimFreeze",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iId"
            }
        ]
    ),
    sig!(
        "RecordLockAI",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "RecordUnlockAI",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!("RecordLockUser", "bool", []),
    sig!("RecordUnLockUser", "bool", []),
    sig!(
        "RecordTimer",
        "bool",
        [NativeParamSig {
            ty: "int",
            name: "iFrames"
        }]
    ),
    sig!(
        "RecordSeekActor",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Actor",
                name: "target"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            },
            NativeParamSig {
                ty: "float",
                name: "fTolerance"
            }
        ]
    ),
    sig!(
        "RecordStopSeek",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "RecordAction",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iID"
            },
            NativeParamSig {
                ty: "int",
                name: "iValue"
            }
        ]
    ),
    sig!(
        "RecordReplaceAnim",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iOriginalAnim"
            },
            NativeParamSig {
                ty: "int",
                name: "iNewAnim"
            }
        ]
    ),
    sig!(
        "RecordRestoreAnim",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iOriginalAnim"
            }
        ]
    ),
    sig!(
        "RecordSpeakPC",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iRemarkID"
            },
            NativeParamSig {
                ty: "int",
                name: "iRemarkVariant"
            }
        ]
    ),
    sig!(
        "RecordTakeCorpse",
        "int",
        [
            NativeParamSig {
                ty: "Actor",
                name: "taker"
            },
            NativeParamSig {
                ty: "Actor",
                name: "corpse"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            }
        ]
    ),
    sig!(
        "RecordMoveIntoBuilding",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "pointBeforeDoor"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            }
        ]
    ),
    sig!(
        "RecordLeaveCorpse",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "ResetAnim",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "RecordStartMobileElement",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iIndex"
        }]
    ),
    sig!(
        "RecordStopMobileElement",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iIndex"
        }]
    ),
    sig!(
        "RecordSpeak",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iRemarkID"
            }
        ]
    ),
    sig!(
        "RecordSeekActorMessage",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "pActor"
            },
            NativeParamSig {
                ty: "Actor",
                name: "pTarget"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            },
            NativeParamSig {
                ty: "float",
                name: "fDistance"
            },
            NativeParamSig {
                ty: "Actor",
                name: "pActorEvent"
            },
            NativeParamSig {
                ty: "int",
                name: "iID"
            }
        ]
    ),
    sig!(
        "RecordSeekActorMessageWithArguments",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "pActor"
            },
            NativeParamSig {
                ty: "Actor",
                name: "pTarget"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            },
            NativeParamSig {
                ty: "float",
                name: "fDistance"
            },
            NativeParamSig {
                ty: "Actor",
                name: "pActorEvent"
            },
            NativeParamSig {
                ty: "int",
                name: "iID"
            },
            NativeParamSig {
                ty: "int",
                name: "iArg1"
            },
            NativeParamSig {
                ty: "int",
                name: "iArg2"
            }
        ]
    ),
    sig!(
        "RecordActivateMobileElement",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iIndex"
        }]
    ),
    sig!(
        "RecordDeactivateMobileElement",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iIndex"
        }]
    ),
    sig!("ThisActor", "Actor", []),
    sig!("GetNumberOfActorsInEngine", "int", []),
    sig!(
        "IsActorAnimation",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorObject",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorCharacter",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorPC",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorNPC",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorSoldier",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorCivilian",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorAnimal",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorCart",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsNull",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorEqual",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "one"
            },
            NativeParamSig {
                ty: "Actor",
                name: "two"
            }
        ]
    ),
    sig!(
        "IsActorDead",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorKO",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorTied",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsActorHS",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "GetActorPosture",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetActorPosture",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iPosture"
            }
        ]
    ),
    sig!(
        "GetActorDirection",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetActorDirection",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iDirection"
            }
        ]
    ),
    sig!(
        "GetActorLocation",
        "Location",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetActorLocation",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "location"
            }
        ]
    ),
    sig!(
        "IsInside",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "location"
            }
        ]
    ),
    sig!(
        "IsInsideBuilding",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Building",
                name: "building"
            }
        ]
    ),
    sig!(
        "UnBlip",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "GetMovementStyle",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "GetCurrentAction",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "InflictPain",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iDamage"
            },
            NativeParamSig {
                ty: "bool",
                name: "bStun"
            }
        ]
    ),
    sig!(
        "StopActor",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "Sees",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actorNPC"
            },
            NativeParamSig {
                ty: "Actor",
                name: "actorTarget"
            }
        ]
    ),
    sig!(
        "EnableViewCone",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!("GetOutlineDisplay", "bool", []),
    sig!(
        "SetOutlineDisplay",
        "void",
        [NativeParamSig {
            ty: "bool",
            name: "bDisplay"
        }]
    ),
    sig!(
        "PrototypeFilterEvent",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "prototype"
            },
            NativeParamSig {
                ty: "Actor",
                name: "actorSource"
            },
            NativeParamSig {
                ty: "int",
                name: "iEvent"
            }
        ]
    ),
    sig!(
        "SendMessage",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actReceiver"
            },
            NativeParamSig {
                ty: "int",
                name: "iMessageCode"
            }
        ]
    ),
    sig!(
        "SendMessageWithArguments",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actReceiver"
            },
            NativeParamSig {
                ty: "int",
                name: "iMessageCode"
            },
            NativeParamSig {
                ty: "int",
                name: "iArgument1"
            },
            NativeParamSig {
                ty: "int",
                name: "iArgument2"
            }
        ]
    ),
    sig!("God", "Actor", []),
    sig!(
        "Select",
        "bool",
        [NativeParamSig {
            ty: "int",
            name: "selectCode"
        }]
    ),
    sig!(
        "Deactivate",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "Activate",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetActionAvailable",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iAction"
            },
            NativeParamSig {
                ty: "bool",
                name: "bAvailable"
            }
        ]
    ),
    sig!(
        "IsActionAvailable",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iAction"
            }
        ]
    ),
    sig!(
        "SetPersistentProperty",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iProperty"
            },
            NativeParamSig {
                ty: "int",
                name: "iAmount"
            }
        ]
    ),
    sig!(
        "GetPersistentProperty",
        "int",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iProperty"
            }
        ]
    ),
    sig!("IsAnyCivilianDead", "bool", []),
    sig!("IsAnyEnemyDead", "bool", []),
    sig!("GetOverallEnemyAlert", "int", []),
    sig!("GetOverallCivilianAlert", "int", []),
    sig!(
        "SetAIAlertStatus",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iStatus"
            }
        ]
    ),
    sig!(
        "GetAIAlertStatus",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetAIState",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iState"
            }
        ]
    ),
    sig!(
        "GetAIState",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetAIAttitude",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iAttitude"
            }
        ]
    ),
    sig!(
        "GetAIAttitude",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetAILevel",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iProperty"
            },
            NativeParamSig {
                ty: "int",
                name: "iLevel"
            }
        ]
    ),
    sig!(
        "StareActor",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Actor",
                name: "actorTarget"
            },
            NativeParamSig {
                ty: "bool",
                name: "bTurnSprite"
            }
        ]
    ),
    sig!(
        "StareLocation",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "locPoint"
            },
            NativeParamSig {
                ty: "bool",
                name: "bTurnSprite"
            }
        ]
    ),
    sig!(
        "AssignPath",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Way",
                name: "myWay"
            }
        ]
    ),
    sig!(
        "AssignPost",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "location"
            },
            NativeParamSig {
                ty: "int",
                name: "iDirection"
            }
        ]
    ),
    sig!(
        "LockAI",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "bool",
                name: "bRememberEvents"
            }
        ]
    ),
    sig!(
        "UnlockAI",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "ForceBattleDecision",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iDecision"
            }
        ]
    ),
    sig!(
        "MakeNoise",
        "void",
        [
            NativeParamSig {
                ty: "Location",
                name: "location"
            },
            NativeParamSig {
                ty: "int",
                name: "iTypeID"
            }
        ]
    ),
    sig!(
        "Freeze",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "bool",
                name: "bFrozen"
            }
        ]
    ),
    sig!(
        "FreezeAll",
        "void",
        [NativeParamSig {
            ty: "bool",
            name: "bFrozen"
        }]
    ),
    sig!(
        "SetPathWalkingStyle",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "NPC"
            },
            NativeParamSig {
                ty: "int",
                name: "i0Walking1Running2Backward"
            }
        ]
    ),
    sig!(
        "GetSoldierRank",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsAnimationActive",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetAnimationState",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "bool",
                name: "bState"
            }
        ]
    ),
    sig!(
        "IsPatchApplied",
        "bool",
        [NativeParamSig {
            ty: "Patch",
            name: "patch"
        }]
    ),
    sig!(
        "ApplyPatch",
        "bool",
        [NativeParamSig {
            ty: "Patch",
            name: "patch"
        }]
    ),
    sig!(
        "ResetPatch",
        "bool",
        [NativeParamSig {
            ty: "Patch",
            name: "patch"
        }]
    ),
    sig!("SuspendAllSoundSources", "bool", []),
    sig!("ResumeAllSoundSources", "bool", []),
    sig!(
        "ActivateSoundSource",
        "bool",
        [NativeParamSig {
            ty: "SoundSource",
            name: "source"
        }]
    ),
    sig!(
        "DeactivateSoundSource",
        "bool",
        [NativeParamSig {
            ty: "SoundSource",
            name: "source"
        }]
    ),
    sig!(
        "DestroySoundSource",
        "bool",
        [NativeParamSig {
            ty: "SoundSource",
            name: "source"
        }]
    ),
    sig!(
        "CleanFromHisBuildingBeforeTeleport",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "CleanFromScriptZoneBeforeTeleport",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "cestLaZone"
            }
        ]
    ),
    sig!(
        "AddToScriptZoneAfterTeleport",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "cestLaZone"
            }
        ]
    ),
    sig!(
        "SetCorpseExistsInBuilding",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "pActor"
        }]
    ),
    sig!(
        "PutActorInBulding",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Building",
                name: "building"
            }
        ]
    ),
    sig!(
        "SetBuildingActive",
        "void",
        [
            NativeParamSig {
                ty: "Building",
                name: "building"
            },
            NativeParamSig {
                ty: "bool",
                name: "bActive"
            }
        ]
    ),
    sig!(
        "GetAnyActorInsideBuilding",
        "Actor",
        [NativeParamSig {
            ty: "Building",
            name: "building"
        }]
    ),
    sig!("NoWhere", "Location", []),
    sig!(
        "GetDistance",
        "int",
        [
            NativeParamSig {
                ty: "Location",
                name: "here"
            },
            NativeParamSig {
                ty: "Location",
                name: "there"
            }
        ]
    ),
    sig!(
        "Rand",
        "int",
        [NativeParamSig {
            ty: "int",
            name: "iMaximum"
        }]
    ),
    sig!(
        "PrintConsole",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iValue"
        }]
    ),
    sig!("GetSizeOfMissionTeam", "int", []),
    sig!(
        "GetPCFromMissionTeam",
        "Actor",
        [NativeParamSig {
            ty: "int",
            name: "ulPC"
        }]
    ),
    sig!(
        "AddPCToMissionTeam",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "RemovePCFromMissionTeam",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!("GetNumberOfObligatoryPCsInMissionTeam", "int", []),
    sig!(
        "GetObligatoryPCFromMissionTeam",
        "Actor",
        [NativeParamSig {
            ty: "int",
            name: "ulPC"
        }]
    ),
    sig!(
        "IsPCObligatoryInMissionTeam",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!("IsMissionTeamValid", "bool", []),
    sig!("GetLastPlayedMission", "int", []),
    sig!("GetNextPlayedMission", "int", []),
    sig!("IsMenToBlazonConversionMode", "bool", []),
    sig!("GetNumberOfBeamMes", "int", []),
    sig!(
        "MoveBeamMe",
        "void",
        [
            NativeParamSig {
                ty: "int",
                name: "iIndex"
            },
            NativeParamSig {
                ty: "Location",
                name: "pLocation"
            }
        ]
    ),
    sig!(
        "SetCompanyNumber",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "pActor"
            },
            NativeParamSig {
                ty: "int",
                name: "iNumber"
            }
        ]
    ),
    sig!(
        "SetAlwaysAttentive",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "bool",
                name: "bYes"
            }
        ]
    ),
    sig!(
        "WinBlazon",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "blazon"
        }]
    ),
    sig!(
        "LoseBlazon",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "blazon"
        }]
    ),
    sig!(
        "SetInvisible",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "bool",
                name: "bHollow"
            }
        ]
    ),
    sig!(
        "IsInvisible",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "IsDoorLockedPC",
        "bool",
        [NativeParamSig {
            ty: "Door",
            name: "door"
        }]
    ),
    sig!(
        "IsDoorUnlockable",
        "bool",
        [NativeParamSig {
            ty: "Door",
            name: "door"
        }]
    ),
    sig!(
        "IsDoorLockedNPCCivilian",
        "bool",
        [NativeParamSig {
            ty: "Door",
            name: "door"
        }]
    ),
    sig!(
        "IsDoorLockedNPCVillain",
        "bool",
        [NativeParamSig {
            ty: "Door",
            name: "door"
        }]
    ),
    sig!(
        "SetDoorLockedPC",
        "void",
        [
            NativeParamSig {
                ty: "Door",
                name: "door"
            },
            NativeParamSig {
                ty: "bool",
                name: "bState"
            }
        ]
    ),
    sig!(
        "SetDoorUnlockable",
        "void",
        [
            NativeParamSig {
                ty: "Door",
                name: "door"
            },
            NativeParamSig {
                ty: "bool",
                name: "bState"
            }
        ]
    ),
    sig!(
        "SetDoorLockedNPCCivilian",
        "void",
        [
            NativeParamSig {
                ty: "Door",
                name: "door"
            },
            NativeParamSig {
                ty: "bool",
                name: "bState"
            }
        ]
    ),
    sig!(
        "SetDoorLockedNPCVillain",
        "void",
        [
            NativeParamSig {
                ty: "Door",
                name: "door"
            },
            NativeParamSig {
                ty: "bool",
                name: "bState"
            }
        ]
    ),
    sig!(
        "SetDoorSpecialAutorisation",
        "void",
        [
            NativeParamSig {
                ty: "Door",
                name: "door"
            },
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "bool",
                name: "bDirect"
            }
        ]
    ),
    sig!(
        "ActivateDoorMouseSector",
        "void",
        [
            NativeParamSig {
                ty: "bool",
                name: "bActive"
            },
            NativeParamSig {
                ty: "Door",
                name: "door"
            }
        ]
    ),
    sig!("ThisScroll", "Actor", []),
    sig!(
        "GetScrollStatus",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "scroll"
        }]
    ),
    sig!(
        "SetScrollStatus",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "scroll"
            },
            NativeParamSig {
                ty: "int",
                name: "iStatus"
            }
        ]
    ),
    sig!(
        "GetCustomCampaignValue",
        "int",
        [NativeParamSig {
            ty: "int",
            name: "iIndex"
        }]
    ),
    sig!(
        "SetCustomCampaignValue",
        "void",
        [
            NativeParamSig {
                ty: "int",
                name: "iIndex"
            },
            NativeParamSig {
                ty: "int",
                name: "iValue"
            }
        ]
    ),
    sig!(
        "GetCustomNPCValue",
        "int",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iIndex"
            }
        ]
    ),
    sig!(
        "SetCustomNPCValue",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iIndex"
            },
            NativeParamSig {
                ty: "int",
                name: "iValue"
            }
        ]
    ),
    sig!(
        "RegisterAsProductionSector",
        "void",
        [
            NativeParamSig {
                ty: "int",
                name: "iType"
            },
            NativeParamSig {
                ty: "Location",
                name: "sector"
            },
            NativeParamSig {
                ty: "int",
                name: "iProductionSpeed"
            }
        ]
    ),
    sig!(
        "AddProductionPoint",
        "void",
        [
            NativeParamSig {
                ty: "int",
                name: "iType"
            },
            NativeParamSig {
                ty: "Location",
                name: "point"
            }
        ]
    ),
    sig!(
        "GetActorForBeamMe",
        "Actor",
        [NativeParamSig {
            ty: "int",
            name: "iIndex"
        }]
    ),
    sig!(
        "DisplayPopupText",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iPopupTextID"
        }]
    ),
    sig!(
        "RecordDisplayPopupText",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iPopupTextID"
        }]
    ),
    sig!(
        "GetNumberOfActorsInSector",
        "int",
        [NativeParamSig {
            ty: "Location",
            name: "loc"
        }]
    ),
    sig!(
        "GetActorInSector",
        "Actor",
        [
            NativeParamSig {
                ty: "Location",
                name: "loc"
            },
            NativeParamSig {
                ty: "int",
                name: "iIndex"
            }
        ]
    ),
    sig!(
        "BitwiseAnd",
        "int",
        [
            NativeParamSig {
                ty: "int",
                name: "i"
            },
            NativeParamSig {
                ty: "int",
                name: "j"
            }
        ]
    ),
    sig!(
        "BitwiseOr",
        "int",
        [
            NativeParamSig {
                ty: "int",
                name: "i"
            },
            NativeParamSig {
                ty: "int",
                name: "j"
            }
        ]
    ),
    sig!(
        "BitwiseXor",
        "int",
        [
            NativeParamSig {
                ty: "int",
                name: "i"
            },
            NativeParamSig {
                ty: "int",
                name: "j"
            }
        ]
    ),
    sig!(
        "HasPCAction",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actPC"
            },
            NativeParamSig {
                ty: "int",
                name: "iActionCode"
            }
        ]
    ),
    sig!(
        "HasAnyPCAction",
        "bool",
        [NativeParamSig {
            ty: "int",
            name: "iActionCode"
        }]
    ),
    sig!("GetRobin", "Actor", []),
    sig!(
        "RecordMoveNear",
        "bool",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "Location",
                name: "location"
            },
            NativeParamSig {
                ty: "int",
                name: "iStyle"
            },
            NativeParamSig {
                ty: "int",
                name: "iTolerance"
            }
        ]
    ),
    sig!(
        "ComputeLocationBetween",
        "Location",
        [
            NativeParamSig {
                ty: "Location",
                name: "locA"
            },
            NativeParamSig {
                ty: "Location",
                name: "locB"
            },
            NativeParamSig {
                ty: "float",
                name: "fLambdaBetweenZeroAndOne"
            }
        ]
    ),
    sig!(
        "DeclareAsCombatTrainer",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "GetRelic",
        "Actor",
        [NativeParamSig {
            ty: "int",
            name: "iID"
        }]
    ),
    sig!("GetNumberOfPCs", "int", []),
    sig!(
        "GetPC",
        "Actor",
        [NativeParamSig {
            ty: "int",
            name: "i"
        }]
    ),
    sig!(
        "AddAsSubordinate",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actChief"
            },
            NativeParamSig {
                ty: "Actor",
                name: "actSubordinate"
            }
        ]
    ),
    sig!(
        "RemoveAllSubordinates",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actChief"
        }]
    ),
    sig!(
        "SwitchToAlertPath",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actSoldier"
        }]
    ),
    sig!(
        "IsActorRider",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actWhoever"
        }]
    ),
    sig!(
        "IsUnblipped",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actWhoever"
        }]
    ),
    sig!(
        "IsBlazonWon",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "blazon"
        }]
    ),
    sig!(
        "AddRepulsivePoint",
        "int",
        [
            NativeParamSig {
                ty: "Location",
                name: "location"
            },
            NativeParamSig {
                ty: "float",
                name: "fRadius"
            },
            NativeParamSig {
                ty: "float",
                name: "fActionRadius"
            },
            NativeParamSig {
                ty: "int",
                name: "iFlags"
            }
        ]
    ),
    sig!(
        "SetViewRadius",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iRadius"
        }]
    ),
    sig!(
        "RecordFreezeAll",
        "void",
        [NativeParamSig {
            ty: "bool",
            name: "bFreeze"
        }]
    ),
    sig!(
        "DeleteRepulsivePoint",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iID"
        }]
    ),
    sig!(
        "SetNPCEmoticon",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actNPC"
            },
            NativeParamSig {
                ty: "int",
                name: "iEmoticonType"
            },
            NativeParamSig {
                ty: "int",
                name: "iTime"
            }
        ]
    ),
    sig!(
        "ConfiscateMoney",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actCapitalist"
        }]
    ),
    sig!(
        "AreAllPCsInside",
        "bool",
        [NativeParamSig {
            ty: "Location",
            name: "location"
        }]
    ),
    sig!(
        "AreAllEnemiesInsideHS",
        "bool",
        [NativeParamSig {
            ty: "Location",
            name: "locZone"
        }]
    ),
    sig!(
        "AddPCToGang",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "AttachScrollToNPC",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actNPC"
            },
            NativeParamSig {
                ty: "Actor",
                name: "scroll"
            }
        ]
    ),
    sig!("AreAllBlazonsWon", "bool", []),
    sig!(
        "IsBonusItemPickedUp",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actItem"
        }]
    ),
    sig!("GetRansomMoney", "int", []),
    sig!(
        "SetRansomMoney",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iRansomMoneyAmount"
        }]
    ),
    sig!("GetDifficultyLevel", "int", []),
    sig!("DisplaySherwoodReport", "void", []),
    sig!(
        "IsActorActive",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "AddFarmerToGang",
        "void",
        [
            NativeParamSig {
                ty: "int",
                name: "iType"
            },
            NativeParamSig {
                ty: "int",
                name: "iExperienceSword"
            },
            NativeParamSig {
                ty: "int",
                name: "iExperienceBow"
            }
        ]
    ),
    sig!(
        "SetExperiences",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iExperienceSword"
            },
            NativeParamSig {
                ty: "int",
                name: "iExperienceBow"
            }
        ]
    ),
    sig!(
        "RecordUnBlip",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "pActor"
        }]
    ),
    sig!(
        "SetPatchAnimationActive",
        "void",
        [
            NativeParamSig {
                ty: "Patch",
                name: "patch"
            },
            NativeParamSig {
                ty: "bool",
                name: "bActive"
            }
        ]
    ),
    sig!("GetNumberOfPCsAlive", "int", []),
    sig!(
        "AreAllPCsAliveInside",
        "bool",
        [NativeParamSig {
            ty: "Location",
            name: "location"
        }]
    ),
    sig!(
        "TransformHandleTargetToTakeTarget",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actTarget"
        }]
    ),
    sig!(
        "IsPCSelected",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actPC"
        }]
    ),
    sig!("GetNumberOfSelectedPCs", "int", []),
    sig!(
        "GetSelectedPC",
        "Actor",
        [NativeParamSig {
            ty: "int",
            name: "iIndex"
        }]
    ),
    sig!("PlayTrapJingle", "void", []),
    sig!(
        "MakePCCrouched",
        "void",
        [NativeParamSig {
            ty: "Actor",
            name: "actPC"
        }]
    ),
    sig!(
        "HasAnyPCActionWhoIsInThisLevelOrCouldMaybeComeFromSherwood",
        "bool",
        [NativeParamSig {
            ty: "int",
            name: "iActionCode"
        }]
    ),
    sig!(
        "LockPatch",
        "void",
        [
            NativeParamSig {
                ty: "Patch",
                name: "patch"
            },
            NativeParamSig {
                ty: "bool",
                name: "bLocked"
            }
        ]
    ),
    sig!(
        "HasAnyActivePCAction",
        "bool",
        [NativeParamSig {
            ty: "int",
            name: "iActionCode"
        }]
    ),
    sig!(
        "GetPCType",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actPC"
        }]
    ),
    sig!(
        "SelectActorPC",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actPCOrGodForAllPCs"
            },
            NativeParamSig {
                ty: "bool",
                name: "bSelectOrUnselect"
            }
        ]
    ),
    sig!(
        "HasAnyActionSelected",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actPC"
        }]
    ),
    sig!(
        "GetActorActionState",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actor"
        }]
    ),
    sig!(
        "SetActorActionState",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actor"
            },
            NativeParamSig {
                ty: "int",
                name: "iActionState"
            }
        ]
    ),
    sig!("SecretAgentsAreBackInSherwood", "bool", []),
    sig!(
        "FadeToBlack",
        "void",
        [NativeParamSig {
            ty: "int",
            name: "iSpeed"
        }]
    ),
    sig!(
        "LinkTargetToFX",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actTarget"
            },
            NativeParamSig {
                ty: "Actor",
                name: "actFX"
            }
        ]
    ),
    sig!(
        "ForbidNPCRemark",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actNPC"
            },
            NativeParamSig {
                ty: "int",
                name: "iRemark"
            },
            NativeParamSig {
                ty: "bool",
                name: "bTrueMeansForbidFalseMeansAllow"
            }
        ]
    ),
    // ── Spellforge / Lua-only natives ──
    sig!(
        "Reveal",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actActor"
        }]
    ),
    sig!(
        "AddObjective",
        "int",
        [
            NativeParamSig {
                ty: "int",
                name: "iObjectiveID"
            },
            NativeParamSig {
                ty: "bool",
                name: "bIsMainObjective"
            },
        ]
    ),
    sig!(
        "CompleteObjective",
        "int",
        [NativeParamSig {
            ty: "int",
            name: "iObjectiveID"
        }]
    ),
    sig!(
        "IsActorOutOfAction",
        "bool",
        [NativeParamSig {
            ty: "Actor",
            name: "actActor"
        }]
    ),
    sig!(
        "SetPatrolShouldRun",
        "void",
        [
            NativeParamSig {
                ty: "Actor",
                name: "actPatrolLeader"
            },
            NativeParamSig {
                ty: "bool",
                name: "bShouldRun"
            },
        ]
    ),
    sig!(
        "SequenceReveal",
        "int",
        [NativeParamSig {
            ty: "Actor",
            name: "actActor"
        }]
    ),
];

pub fn native_signature_by_index(index: u32) -> Option<&'static NativeSignature> {
    NativeFn::try_from(index).ok()?;
    native_signature_by_name(native_name(index))
}

pub fn native_signature_by_name(name: &str) -> Option<&'static NativeSignature> {
    NATIVE_SIGNATURES.iter().find(|sig| sig.name == name)
}
