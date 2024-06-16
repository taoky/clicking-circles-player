﻿// Original source file (modified by kabii) Copyright (c) ppy Pty Ltd <contact@ppy.sh>. Licensed under the MIT Licence.
using Realms;

namespace CollectionDowngrader.LazerSchema
{
    public class RealmNamedFileUsage : EmbeddedObject
    {
        public RealmFile File { get; set; } = null!;
        public string Filename { get; set; } = null!;

    }
}
