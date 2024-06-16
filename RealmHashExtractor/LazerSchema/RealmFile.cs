﻿// Original source file (modified by kabii) Copyright (c) ppy Pty Ltd <contact@ppy.sh>. Licensed under the MIT Licence.
using Realms;

namespace CollectionDowngrader.LazerSchema
{
    [MapTo("File")]
    public class RealmFile : RealmObject
    {
        [PrimaryKey]
        public string Hash { get; set; } = string.Empty;
    }
}
